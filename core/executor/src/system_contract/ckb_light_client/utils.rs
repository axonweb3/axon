use common_config_parser::types::ConfigRocksDB;
use protocol::{
    types::{Apply, ApplyBackend, Backend, Basic, H160, H256, U256},
    ProtocolResult,
};
use std::{path::Path, sync::Arc};

use crate::{
    system_contract::{
        ckb_light_client::{CkbLightClientContract, CELL_ROOT_KEY, CURRENT_CELL_ROOT, TRIE_DB},
        error::SystemScriptError,
        image_cell::{exec::save_cells, utils::always_success_script_deploy_cell},
        trie_db::RocksTrieDB,
        SystemContract,
    },
    MPTTrie,
};

const DEFAULT_CACHE_SIZE: usize = 20;

pub fn init<P: AsRef<Path>, B: Backend + ApplyBackend>(
    path: P,
    config: ConfigRocksDB,
    mut backend: B,
) {
    TRIE_DB.get_or_init(|| {
        Arc::new(
            RocksTrieDB::new(path, config, DEFAULT_CACHE_SIZE)
                .expect("[image cell] new rocksdb error"),
        )
    });

    let current_cell_root = backend.storage(CkbLightClientContract::ADDRESS, *CELL_ROOT_KEY);
    if current_cell_root.is_zero() {
        let mut mpt = get_mpt().unwrap();
        // todo need refactoring
        save_cells(&mut mpt, vec![always_success_script_deploy_cell()], 0).unwrap();
        return update_mpt_root(
            &mut backend,
            mpt.commit().unwrap(),
            CkbLightClientContract::ADDRESS,
        );
    }

    CURRENT_CELL_ROOT.store(Arc::new(current_cell_root));
}

pub fn get_mpt() -> ProtocolResult<MPTTrie<RocksTrieDB>> {
    let trie_db = match TRIE_DB.get() {
        Some(db) => db,
        None => return Err(SystemScriptError::TrieDbNotInit.into()),
    };

    let root = **CURRENT_CELL_ROOT.load();

    if root == H256::default() {
        Ok(MPTTrie::new(Arc::clone(trie_db)))
    } else {
        match MPTTrie::from_root(root, Arc::clone(trie_db)) {
            Ok(m) => Ok(m),
            Err(e) => Err(SystemScriptError::RestoreMpt(e.to_string()).into()),
        }
    }
}

pub fn update_mpt_root<B: Backend + ApplyBackend>(backend: &mut B, root: H256, address: H160) {
    let account = backend.basic(address);
    CURRENT_CELL_ROOT.swap(Arc::new(root));
    backend.apply(
        vec![Apply::Modify {
            address:       CkbLightClientContract::ADDRESS,
            basic:         Basic {
                balance: account.balance,
                nonce:   account.nonce + U256::one(),
            },
            code:          None,
            storage:       vec![(*CELL_ROOT_KEY, root)],
            reset_storage: false,
        }],
        vec![],
        false,
    );
}
