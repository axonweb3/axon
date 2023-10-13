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
    spec::{ChainSpec, ChainSpecValueParser, HardforkName},
    Config, ConfigValueParser,
};
use core_executor::{
    system_contract::metadata::{
        encode_consensus_config, segment::EpochSegment, CONSENSUS_CONFIG, EPOCH_SEGMENT_KEY,
        HARDFORK_KEY,
    },
    AxonExecutorApplyAdapter, MetadataHandle,
};
use protocol::{
    codec::{hex_decode, ProtocolCodec as _},
    tokio,
    trie::{MemoryDB, PatriciaTrie, Trie as _},
    types::{
        Bloom, BloomInput, HardforkInfo, HardforkInfoInner, Header, Metadata, Proposal, RichBlock,
        H256, RLP_EMPTY_LIST, RLP_NULL,
    },
};

use crate::{components::chain_spec::ChainSpecExt as _, execute_genesis, DatabaseGroup};

const DEV_CONFIG_DIR: &str = "../../devtools/chain";

struct TestCase<'a> {
    chain_name:         &'a str,
    config_file:        &'a str,
    chain_spec_file:    &'a str,
    input_genesis_hash: &'a str,
    genesis_state_root: &'a str,
}

const TESTCASES: &[TestCase] = &[
    TestCase {
        chain_name:         "single_node",
        config_file:        "config.toml",
        chain_spec_file:    "specs/single_node/chain-spec.toml",
        input_genesis_hash: "0x3428ea0eb8fb33f193e4680772ec6f68279c24bb7f135a8a3f790af6539fc100",
        genesis_state_root: "0xe7799df5d8256add895412df5889c4ae07bb5c0d7fa438801b910cfef9d5dc37",
    },
    TestCase {
        chain_name:         "multi_nodes",
        config_file:        "nodes/node_1.toml",
        chain_spec_file:    "specs/multi_nodes/chain-spec.toml",
        input_genesis_hash: "0xa2559187fdb9bc172ee82b40c9f4f166d56d0ba9dc2fc47538e7d95a8d641575",
        genesis_state_root: "0x04b03f59e424ab276cd3a8c54e9f3d51c8f1c9fc64c4b2fda7accd2ddffb46b4",
    },
    TestCase {
        chain_name:         "multi_nodes_short_epoch_len",
        config_file:        "nodes/node_1.toml",
        chain_spec_file:    "specs/multi_nodes_short_epoch_len/chain-spec.toml",
        input_genesis_hash: "0xa710efc9621394b5138a0e9a87c7b8586a88f4d26059bcb06ee76cc5e0bf12b0",
        genesis_state_root: "0x3a9030d8e60478866e618a92c5137e02c1f3dc0b20dc5b51f4e720d7a7a58a0b",
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

    let path_block = tmp_dir.path().join("block");
    let db_group = DatabaseGroup::new(
        &config.rocksdb,
        path_block,
        true,
        config.executor.triedb_cache_size,
    )
    .expect("initialize databases");

    let partial_genesis = chain_spec.generate_genesis_block();
    let genesis = execute_genesis(partial_genesis, &chain_spec, &db_group)
        .await
        .expect("complete genesis block");

    assert!(genesis.txs.is_empty());
    assert!(genesis.block.tx_hashes.is_empty());

    println!("checking genesis hash");
    check_hashes_via_str(
        case.chain_name,
        "input genesis hash",
        case.input_genesis_hash,
        genesis.block.header.hash(),
    );

    println!("checking state root");
    check_hashes_via_str(
        case.chain_name,
        "genesis state root",
        case.genesis_state_root,
        genesis.block.header.state_root,
    );

    check_hashes(
        case.chain_name,
        "genesis transactions root",
        genesis.block.header.transactions_root,
        RLP_NULL,
    );

    check_hashes(
        case.chain_name,
        "genesis signed transactions hash",
        genesis.block.header.signed_txs_hash,
        RLP_EMPTY_LIST,
    );

    println!("checking receipts hash");
    check_hashes(
        case.chain_name,
        "genesis receipts root",
        genesis.block.header.receipts_root,
        RLP_NULL,
    );

    let logs: Vec<Bloom> = Default::default();
    let expected_log_bloom = Bloom::from(BloomInput::Raw(rlp::encode_list(&logs).as_ref()));
    assert_eq!(
        genesis.block.header.log_bloom, expected_log_bloom,
        "log bloom in genesis of chain {} should be empty since no transactions, \
        expect {expected_log_bloom:#x}, actual {:#x}",
        case.chain_name, genesis.block.header.log_bloom,
    );

    assert!(
        genesis.block.header.gas_used.is_zero(),
        "gas used in genesis of chain {} should be zero since no transactions, actual {}",
        case.chain_name,
        genesis.block.header.gas_used,
    );

    println!("checking state");
    check_state(&chain_spec, &genesis.block.header, &db_group);

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

fn check_hashes_via_str(chain: &str, name: &str, expected_str: &str, actual: H256) {
    let expected = H256::from_str(expected_str)
        .unwrap_or_else(|err| panic!("failed to parse hash {name} of chain {chain} since {err}"));
    check_hashes(chain, name, expected, actual);
}

fn check_hashes(chain: &str, name: &str, expected: H256, actual: H256) {
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
    let (inner_0, config_0) = metadata_0.into_part();
    let (inner_1, config_1) = metadata_1.into_part();

    memory_mpt
        .insert(
            inner_0.epoch.to_be_bytes().to_vec(),
            inner_0.encode().unwrap().to_vec(),
        )
        .unwrap();
    memory_mpt
        .insert(
            CONSENSUS_CONFIG.as_bytes().to_vec(),
            encode_consensus_config(
                H256::from_low_u64_be((HardforkName::None as u64).to_be()),
                config_0,
            )
            .unwrap(),
        )
        .unwrap();
    memory_mpt
        .insert(
            inner_1.epoch.to_be_bytes().to_vec(),
            inner_1.encode().unwrap().to_vec(),
        )
        .unwrap();
    memory_mpt
        .insert(
            CONSENSUS_CONFIG.as_bytes().to_vec(),
            encode_consensus_config(
                H256::from_low_u64_be((HardforkName::None as u64).to_be()),
                config_1,
            )
            .unwrap(),
        )
        .unwrap();

    let info = HardforkInfoInner {
        flags:        H256::from_low_u64_be(HardforkName::all().to_be()),
        block_number: 0,
    };
    let hardfork = HardforkInfo { inner: vec![info] }
        .encode()
        .unwrap()
        .to_vec();

    memory_mpt
        .insert(HARDFORK_KEY.as_bytes().to_vec(), hardfork)
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
