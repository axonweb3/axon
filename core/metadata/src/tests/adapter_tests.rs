use std::fs::{self, File};
use std::io::BufReader;
use std::sync::Arc;

use rand::random;

use core_executor::adapter::{MPTTrie, RocksTrieDB};
use core_executor::{EVMExecutorAdapter, EvmExecutor};
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use protocol::traits::{CommonStorage, Context, Executor, Storage};
use protocol::types::{
    Account, Address, Hash, Hasher, Hex, Metadata, Proposal, RichBlock, H160, H256, NIL_DATA,
    RLP_NULL,
};
use protocol::{codec::ProtocolCodec, tokio};

use crate::metadata_abi as abi;
use crate::{MetadataAdapterImpl, MetadataController};

const GENESIS_PATH: &str = "../../devtools/chain/genesis.json";

fn metadata_address() -> H160 {
    H160::from_slice(&Hex::decode("".to_string()).unwrap())
}

struct TestHandle {
    storage:    Arc<ImplStorage<RocksAdapter>>,
    trie_db:    Arc<RocksTrieDB>,
    state_root: H256,
}

impl Drop for TestHandle {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all("./free-space");
    }
}

impl TestHandle {
    pub async fn new() -> Self {
        let storage_adapter = RocksAdapter::new("./free-space/rocks", 1024).unwrap();
        let trie_db = RocksTrieDB::new("./free-space/trie", 1024, 50).unwrap();

        let mut handle = TestHandle {
            storage:    Arc::new(ImplStorage::new(Arc::new(storage_adapter))),
            trie_db:    Arc::new(trie_db),
            state_root: H256::default(),
        };

        handle.load_genesis().await;
        handle
    }

    async fn load_genesis(&mut self) {
        let reader = BufReader::new(File::open(GENESIS_PATH).unwrap());
        let genesis: RichBlock = serde_json::from_reader(reader).unwrap();
        let distribute_address =
            Address::from_hex("0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1").unwrap();
        let distribute_account = Account {
            nonce:        0u64.into(),
            balance:      32000001100000000000u128.into(),
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        };

        let mut mpt = MPTTrie::new(Arc::clone(&self.trie_db));

        mpt.insert(
            distribute_address.as_slice(),
            distribute_account.encode().unwrap().as_ref(),
        )
        .unwrap();

        let proposal = Proposal::from(genesis.block.clone());
        let executor = EvmExecutor::default();
        let mut backend = EVMExecutorAdapter::from_root(
            mpt.commit().unwrap(),
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            proposal.into(),
        )
        .unwrap();

        let resp = executor.exec(&mut backend, genesis.txs.clone());
        println!("{:?}", resp);
        self.state_root = resp.state_root;

        self.storage
            .update_latest_proof(Context::new(), genesis.block.header.proof.clone())
            .await
            .unwrap();
        self.storage
            .insert_block(Context::new(), genesis.block.clone())
            .await
            .unwrap();
        self.storage
            .insert_transactions(
                Context::new(),
                genesis.block.header.number,
                genesis.txs.clone(),
            )
            .await
            .unwrap();
    }

    pub fn metadata_controller(
        &self,
        epoch_len: u64,
    ) -> MetadataController<MetadataAdapterImpl<ImplStorage<RocksAdapter>, RocksTrieDB>> {
        let adapter =
            MetadataAdapterImpl::new(Arc::clone(&self.storage), Arc::clone(&self.trie_db));
        MetadataController::new(Arc::new(adapter), Default::default(), epoch_len)
    }
}

// #[test]
// fn test() {
// 	use ethers::core::abi::AbiEncode;

//     let r =
// BufReader::new(File::open("../../devtools/chain/metadata.json").unwrap());
//     let metadata: Metadata = serde_json::from_reader(r).unwrap();
// 	let call = abi::MetadataContractCalls::AppendMetadata(abi::AppendMetadataCall
// { 		metadata: metadata.into(),
// 	});
// 	let raw_call = call.encode();
// 	println!("{:?}", raw_call);

// 	use protocol::types::{TransactionAction, Hex};

// 	let ac = TransactionAction::Call(H256::from_slice(&Hex::decode("
// 0xc34393e6a797d2b4e2aabbc7b9dc8bde1db42410d304b5e78c2ff843134e15e0".
// to_string()).unwrap()).into()); 	println!("{:?}",
// serde_json::to_string_pretty(&ac).unwrap()); }

#[tokio::test(flavor = "multi_thread")]
async fn test_1() {
    let handle = TestHandle::new().await;
    let ctl = handle.metadata_controller(100);
}
