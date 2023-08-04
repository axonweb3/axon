use std::{
    convert::AsRef,
    env::set_current_dir,
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use common_config_parser::{parse_file, types::Config};
use core_executor::{MPTTrie, RocksTrieDB};
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use protocol::{
    codec::hex_decode,
    tokio,
    types::{RichBlock, H256},
};

use crate::{execute_transactions, insert_accounts};

const DEV_CONFIG_DIR: &str = "../../devtools/chain";

const GENESIS_FILE: &str = "genesis_single_node.json";
const CONFIG_FILE: &str = "config.toml";

const GENESIS_HASH: &str = "0x69eca8420bb4b072d732db96260a656f0e10201c6841b215a8ed107681e17d1c";
const GENESIS_STATE_ROOT: &str =
    "0x65f57a6a666e656de33ed68957e04b35b3fe1b35a90f6eafb6f283c907dc3d77";
const GENESIS_RECEIPTS_ROOT: &str =
    "0x0abcdb8fd7acc8c71f28a16c3095fdafca0171f371076650152b1c356a1bccad";

#[test]
fn decode_genesis() {
    let raw = fs::read("../../devtools/chain/genesis_single_node.json").unwrap();
    assert!(serde_json::from_slice::<RichBlock>(&raw).is_ok());
}

#[test]
fn decode_type_id() {
    let type_id_str = "c0810210210c06ba233273e94d7fc89b00a705a07fdc0ae4c78e4036582ff336";
    assert!(hex_decode(type_id_str).is_ok());
}

#[tokio::test(flavor = "multi_thread")]
async fn genesis_hash_for_dev_chain() {
    let dev_config_dir = PathBuf::from_str(DEV_CONFIG_DIR).expect("read dev config dir");
    let tmp_dir = tempfile::tempdir().unwrap_or_else(|err| {
        panic!("failed to create temporary directory since {err:?}");
    });
    let tmp_dir_path = tmp_dir.path();

    copy_dir(dev_config_dir, tmp_dir_path).expect("configs copied");
    set_current_dir(tmp_dir_path).expect("change work directory");

    let config: Config = {
        let config_path = tmp_dir_path.join(CONFIG_FILE);
        parse_file(config_path, false).expect("parse config file")
    };
    let genesis: RichBlock = {
        let genesis_path = tmp_dir_path.join(GENESIS_FILE);
        parse_file(genesis_path, true).expect("parse genesis file")
    };

    let expected_genesis_hash = H256::from_str(GENESIS_HASH).expect("expected genesis hash");
    check_hashes(
        "genesis hash",
        expected_genesis_hash,
        genesis.block.header.hash(),
    );

    let storage = {
        let path_block = tmp_dir.path().join("block");
        let rocks_adapter = Arc::new(
            RocksAdapter::new(path_block, config.rocksdb.clone()).expect("temporary block storage"),
        );
        let impl_storage = ImplStorage::new(rocks_adapter, config.rocksdb.cache_size);
        Arc::new(impl_storage)
    };

    let trie_db = {
        let path_state = tmp_dir.path().join("state");
        let trie_db = RocksTrieDB::new(
            path_state,
            config.rocksdb.clone(),
            config.executor.triedb_cache_size,
        )
        .expect("temporary trie db");
        Arc::new(trie_db)
    };

    let state_root = {
        let mut mpt = MPTTrie::new(Arc::clone(&trie_db));
        insert_accounts(&mut mpt, &config.accounts).expect("insert accounts");
        mpt.commit().expect("mpt commit")
    };

    let path_metadata = tmp_dir_path.join("metadata");
    let resp = execute_transactions(
        &genesis,
        state_root,
        &trie_db,
        &storage,
        path_metadata,
        &config.rocksdb,
    )
    .expect("execute transactions");

    let expected_state_root = H256::from_str(GENESIS_STATE_ROOT).expect("expected genesis hash");
    check_hashes("genesis state root", expected_state_root, resp.state_root);
    let expected_receipts_root =
        H256::from_str(GENESIS_RECEIPTS_ROOT).expect("expected genesis hash");
    check_hashes(
        "genesis receipts root",
        expected_receipts_root,
        resp.receipt_root,
    );
}

fn check_hashes(name: &str, expected: H256, actual: H256) {
    assert_eq!(
        expected, actual,
        "{name} of dev chain is changed, expect {expected:#x}, but got {actual:#x}",
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
