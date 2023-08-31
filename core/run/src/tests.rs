use std::{
    convert::AsRef,
    env::{current_dir, set_current_dir},
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{builder::TypedValueParser as _, Command};

use common_config_parser::types::{
    spec::{ChainSpec, ChainSpecValueParser},
    Config, ConfigValueParser,
};
use protocol::{
    codec::hex_decode,
    tokio,
    types::{RichBlock, H256},
};

use crate::{execute_transactions, DatabaseGroup};

const DEV_CONFIG_DIR: &str = "../../devtools/chain";

struct TestCase<'a> {
    chain_name:            &'a str,
    config_file:           &'a str,
    chain_spec_file:       &'a str,
    input_genesis_hash:    &'a str,
    genesis_state_root:    &'a str,
    genesis_receipts_root: &'a str,
}

const TESTCASES: &[TestCase] = &[
    TestCase {
        chain_name:            "single_node",
        config_file:           "config.toml",
        chain_spec_file:       "specs/single_node/chain-spec.toml",
        input_genesis_hash:    "0x2043f690fc6e086c6940a083072a82dee16c18a4c4afaf6f4e1c7a585fae2543",
        genesis_state_root:    "0x601bd874d41eb9adb32021ee3ab934e0481065c58abfe7e757e33fb01be18dd5",
        genesis_receipts_root: "0x8544b530238201f1620b139861a6841040b37f78f8bdae8736ef5cec474e979b",
    },
    TestCase {
        chain_name:            "multi_nodes",
        config_file:           "nodes/node_1.toml",
        chain_spec_file:       "specs/multi_nodes/chain-spec.toml",
        input_genesis_hash:    "0x5e5c47725bb1face59965a326b1d69e1ada1892da2e2f53c4520ed5da3d88d59",
        genesis_state_root:    "0xc36f75519a047fec6a34c7be5dfca783a40eafa0d7418ad7b3ba99ad9c2dc655",
        genesis_receipts_root: "0x8544b530238201f1620b139861a6841040b37f78f8bdae8736ef5cec474e979b",
    },
    TestCase {
        chain_name:            "multi_nodes_short_epoch_len",
        config_file:           "nodes/node_1.toml",
        chain_spec_file:       "specs/multi_nodes_short_epoch_len/chain-spec.toml",
        input_genesis_hash:    "0x2043f690fc6e086c6940a083072a82dee16c18a4c4afaf6f4e1c7a585fae2543",
        genesis_state_root:    "0x42886558baab8a3c310d5a8313398e5f353cc4f8192838b578c857a329e9bb65",
        genesis_receipts_root: "0x8544b530238201f1620b139861a6841040b37f78f8bdae8736ef5cec474e979b",
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
    for case in TESTCASES {
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
    let genesis = chain_spec.genesis.build_rich_block();

    check_hashes(
        case.chain_name,
        "input genesis hash",
        case.input_genesis_hash,
        genesis.block.header.hash(),
    );

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

    let resp = execute_transactions(&genesis, &db_group, &chain_spec.accounts)
        .expect("execute transactions");

    check_hashes(
        case.chain_name,
        "genesis state root",
        case.genesis_state_root,
        resp.state_root,
    );
    check_hashes(
        case.chain_name,
        "genesis receipts root",
        case.genesis_receipts_root,
        resp.receipt_root,
    );

    set_current_dir(current_dir).expect("change back to original work directory");
}

fn check_hashes(chain: &str, name: &str, expected_str: &str, actual: H256) {
    let expected = H256::from_str(expected_str)
        .unwrap_or_else(|err| panic!("failed to parse hash {name} of chain {chain} since {err}"));
    assert_eq!(
        expected, actual,
        "hash {name} of chain {chain} is changed, expect {expected:#x}, but got {actual:#x}",
    );
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
