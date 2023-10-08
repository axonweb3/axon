use std::{
    collections::BTreeMap,
    convert::AsRef,
    env::{current_dir, set_current_dir},
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use clap::{builder::TypedValueParser as _, Command};
use hasher::HasherKeccak;

use common_config_parser::types::{
    spec::{ChainSpec, ChainSpecValueParser, PrivateKeyFileValueParser},
    Config, ConfigValueParser,
};
use common_crypto::Secp256k1RecoverablePrivateKey;
use core_executor::{
    system_contract::metadata::{segment::EpochSegment, EPOCH_SEGMENT_KEY},
    AxonExecutorApplyAdapter, MetadataHandle,
};
use protocol::{
    codec::{hex_decode, ProtocolCodec as _},
    tokio,
    trie::{MemoryDB, PatriciaTrie, Trie as _},
    types::{Header, Metadata, Proposal, RichBlock, H256},
};

use crate::{
    components::chain_spec::ChainSpecExt as _, execute_genesis_transactions, DatabaseGroup,
};

const DEV_CONFIG_DIR: &str = "../../devtools/chain";

struct TestCase<'a> {
    chain_name:            &'a str,
    config_file:           &'a str,
    chain_spec_file:       &'a str,
    key_file:              &'a str,
    input_genesis_hash:    &'a str,
    genesis_state_root:    &'a str,
    genesis_receipts_root: &'a str,
}

const TESTCASES: &[TestCase] = &[
    TestCase {
        chain_name:            "single_node",
        config_file:           "config.toml",
        chain_spec_file:       "specs/single_node/chain-spec.toml",
        key_file:              "debug.key",
        input_genesis_hash:    "0x4e06dc4a01178db42c029f7d65f65a5763702a21082cfcb626c6c41054a7a276",
        genesis_state_root:    "0x2f1e8e50d5ab97af96fdb5d6de8e691e5bb80f46f2c98c4133d265bd8b60de61",
        genesis_receipts_root: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    },
    TestCase {
        chain_name:            "multi_nodes",
        config_file:           "nodes/node_1.toml",
        chain_spec_file:       "specs/multi_nodes/chain-spec.toml",
        key_file:              "debug.key",
        input_genesis_hash:    "0xf16db25ca1a0cff5339d76e9802c75c43faac35ee4a9294a51234b167c69159f",
        genesis_state_root:    "0xf684cbec490eb5b8a07b80f369f3bf87f05ec73494b869111010a6ad6fa89894",
        genesis_receipts_root: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    },
    TestCase {
        chain_name:            "multi_nodes_short_epoch_len",
        config_file:           "nodes/node_1.toml",
        chain_spec_file:       "specs/multi_nodes_short_epoch_len/chain-spec.toml",
        key_file:              "debug.key",
        input_genesis_hash:    "0x4e06dc4a01178db42c029f7d65f65a5763702a21082cfcb626c6c41054a7a276",
        genesis_state_root:    "0xa5e1e7ac3e03f7dc26cc93ab69c0ec49e591cbdaa7694c75682745c40bfca468",
        genesis_receipts_root: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    },
];

#[test]
fn decode_genesis() {
    let raw = fs::read("../../tests/data/genesis.json").unwrap();
    assert!(serde_json::from_slice::<RichBlock>(&raw).is_ok());
}

#[test]
fn decode_type_id() {
    let type_id_str = "c0810210210c06ba233273e94d7fc89b00a705a07fdc0ae4c78e4036582ff336";
    assert!(hex_decode(type_id_str).is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn genesis_data_for_dev_chain() {
    for case in TESTCASES.iter() {
        println!("======Test case {:?}======", case.chain_name);
        check_genesis_data(case).await;
    }
}

async fn check_genesis_data<'a>(case: &TestCase<'a>) {
    let dev_config_dir = PathBuf::from_str(DEV_CONFIG_DIR).expect("read dev config dir");
    let tmp_dir = tempfile::tempdir().unwrap_or_else(|err| {
        panic!("failed to create temporary directory since {err:?}");
    });
    let tmp_dir_path = tmp_dir.path();
    let command = Command::new("dummy-command");

    copy_dir(dev_config_dir, tmp_dir_path).expect("configs copied");
    let current_dir = current_dir().expect("get current directory");
    set_current_dir(tmp_dir_path).expect("change work directory");

    let config: Config = {
        let config_path = tmp_dir_path.join(case.config_file);
        ConfigValueParser
            .parse_ref(&command, None, config_path.as_os_str())
            .expect("parse config file")
    };
    let chain_spec: ChainSpec = {
        let chain_spec_path = tmp_dir_path.join(case.chain_spec_file);
        ChainSpecValueParser
            .parse_ref(&command, None, chain_spec_path.as_os_str())
            .expect("parse chain-spec file")
    };
    let key: Secp256k1RecoverablePrivateKey = {
        let key_file_path = tmp_dir_path.join(case.key_file);
        let key_data = PrivateKeyFileValueParser
            .parse_ref(&command, None, key_file_path.as_os_str())
            .expect("parse key file");
        Secp256k1RecoverablePrivateKey::try_from(key_data.as_ref()).expect("load key data")
    };
    let genesis = chain_spec.generate_genesis_block(key);

    assert!(genesis.txs.is_empty());

    println!("checking genesis hash");
    check_hashes(
        case.chain_name,
        "input genesis hash",
        case.input_genesis_hash,
        genesis.block.header.hash(),
    );

    assert!(genesis.txs.is_empty());

    for (i, (block_cached, tx)) in genesis
        .block
        .tx_hashes
        .iter()
        .zip(genesis.txs.iter())
        .enumerate()
    {
        let tx_cached = tx.transaction.hash;
        assert_eq!(
            *block_cached, tx_cached,
            "check hash of tx[{i}], in-block: {block_cached:#x}, tx-cached: {tx_cached:#x}",
        );
        let calculated = tx.transaction.clone().calc_hash().hash;
        assert_eq!(
            tx_cached, calculated,
            "check hash of tx[{i}], cached: {tx_cached:#x}, calculated: {calculated:#x}",
        );
    }
    let path_block = tmp_dir.path().join("block");
    let db_group = DatabaseGroup::new(
        &config.rocksdb,
        path_block,
        true,
        config.executor.triedb_cache_size,
    )
    .expect("initialize databases");

    let metadata_0 = chain_spec.params.clone();
    let metadata_1 = {
        let mut tmp = metadata_0.clone();
        tmp.epoch = metadata_0.epoch + 1;
        tmp.version.start = metadata_0.version.end + 1;
        tmp.version.end = tmp.version.start + metadata_0.version.end - 1;
        tmp
    };
    let resp = execute_genesis_transactions(&genesis, &db_group, &chain_spec.accounts, &[
        metadata_0, metadata_1,
    ])
    .expect("execute transactions");

    let mut header = genesis.block.header.clone();
    header.state_root = resp.state_root;
    header.receipts_root = resp.receipt_root;

    println!("checking state root");
    check_hashes(
        case.chain_name,
        "genesis state root",
        case.genesis_state_root,
        resp.state_root,
    );

    println!("checking receipts hash");
    check_hashes(
        case.chain_name,
        "genesis receipts root",
        case.genesis_receipts_root,
        resp.receipt_root,
    );

    println!("checking state");
    check_state(&chain_spec, &header, &db_group);

    set_current_dir(current_dir).expect("change back to original work directory");
}

fn check_state(spec: &ChainSpec, genesis_header: &Header, db_group: &DatabaseGroup) {
    let backend = AxonExecutorApplyAdapter::from_root(
        genesis_header.state_root,
        db_group.trie_db(),
        db_group.storage(),
        Proposal::new_without_state_root(genesis_header).into(),
    )
    .unwrap();

    let metadata_0 = spec.params.clone();
    let metadata_1 = {
        let mut tmp = metadata_0.clone();
        tmp.epoch = metadata_0.epoch + 1;
        tmp.version.start = metadata_0.version.end + 1;
        tmp.version.end = tmp.version.start + metadata_0.version.end - 1;
        tmp
    };
    let handle = MetadataHandle::new(backend.get_metadata_root());

    assert_eq!(
        backend.get_metadata_root().as_bytes(),
        generate_memory_mpt_root(metadata_0.clone(), metadata_1.clone())
    );

    assert_metadata(metadata_0, handle.get_metadata_by_epoch(0).unwrap());
    assert_metadata(metadata_1, handle.get_metadata_by_epoch(1).unwrap());
}

fn check_hashes(chain: &str, name: &str, expected_str: &str, actual: H256) {
    let expected = H256::from_str(expected_str)
        .unwrap_or_else(|err| panic!("failed to parse hash {name} of chain {chain} since {err}"));
    assert_eq!(
        expected, actual,
        "hash {name} of chain {chain} is changed, expect {expected:#x}, but got {actual:#x}",
    );
}

fn assert_metadata(left: Metadata, right: Metadata) {
    assert_eq!(left.version, right.version);
    assert_eq!(left.epoch, right.epoch);
    assert_eq!(left.verifier_list, right.verifier_list);
    assert_eq!(left.consensus_config, right.consensus_config);
}

fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn generate_memory_mpt_root(metadata_0: Metadata, metadata_1: Metadata) -> Vec<u8> {
    let metadata_0 = sort_metadata(metadata_0);
    let metadata_1 = sort_metadata(metadata_1);
    let mut memory_mpt = PatriciaTrie::new(
        Arc::new(MemoryDB::new(false)),
        Arc::new(HasherKeccak::new()),
    );
    let mut seg = EpochSegment::new();

    memory_mpt
        .insert(EPOCH_SEGMENT_KEY.as_bytes().to_vec(), seg.as_bytes())
        .unwrap();
    seg.append_endpoint(metadata_0.version.end).unwrap();
    memory_mpt
        .insert(EPOCH_SEGMENT_KEY.as_bytes().to_vec(), seg.as_bytes())
        .unwrap();
    seg.append_endpoint(metadata_1.version.end).unwrap();
    memory_mpt
        .insert(EPOCH_SEGMENT_KEY.as_bytes().to_vec(), seg.as_bytes())
        .unwrap();

    memory_mpt
        .insert(
            metadata_0.epoch.to_be_bytes().to_vec(),
            metadata_0.encode().unwrap().to_vec(),
        )
        .unwrap();
    memory_mpt
        .insert(
            metadata_1.epoch.to_be_bytes().to_vec(),
            metadata_1.encode().unwrap().to_vec(),
        )
        .unwrap();
    memory_mpt.root().unwrap()
}

fn sort_metadata(mut metadata: Metadata) -> Metadata {
    let map = metadata
        .verifier_list
        .iter()
        .map(|v| (v.address, 0u64))
        .collect::<BTreeMap<_, _>>();
    metadata.propose_counter = map.into_iter().map(Into::into).collect();
    metadata
}
