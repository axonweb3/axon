mod controller;

use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use ethers_core::abi::AbiEncode;

use core_executor::adapter::{MPTTrie, RocksTrieDB};
use core_executor::{AxonExecutor, AxonExecutorAdapter};
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use protocol::codec::ProtocolCodec;
use protocol::traits::{CommonStorage, Context, Executor, Storage};
use protocol::types::{
    Account, Address, Bytes, Eip1559Transaction, Header, Metadata, Proposal, Public, RichBlock,
    SignatureComponents, SignedTransaction, TransactionAction, UnsignedTransaction,
    UnverifiedTransaction, H160, H256, NIL_DATA, RLP_NULL, U256,
};

use crate::{calc_epoch, metadata_abi as abi, MetadataAdapterImpl, MetadataController, EPOCH_LEN};

const GENESIS_PATH: &str = "../../devtools/chain/genesis_single_node.json";
pub const METADATA_CONTRACT_ADDRESS: H160 = H160([
    176, 13, 97, 107, 130, 12, 57, 97, 158, 226, 158, 81, 68, 208, 34, 108, 248, 181, 193, 90,
]);

struct TestHandle {
    storage:    Arc<ImplStorage<RocksAdapter>>,
    trie_db:    Arc<RocksTrieDB>,
    state_root: H256,
}

impl TestHandle {
    pub async fn new(salt: u64) -> Self {
        let path = "./free-space/".to_string();
        let storage_adapter = RocksAdapter::new(
            path.clone() + &salt.to_string() + "/rocks",
            Default::default(),
        )
        .unwrap();
        let trie_db =
            RocksTrieDB::new(path + &salt.to_string() + "/trie", Default::default(), 50).unwrap();

        let mut handle = TestHandle {
            storage:    Arc::new(ImplStorage::new(Arc::new(storage_adapter), 10)),
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

        let proposal = Proposal::from(&genesis.block);
        let executor = AxonExecutor::default();
        let mut backend = AxonExecutorAdapter::from_root(
            mpt.commit().unwrap(),
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            proposal.into(),
        )
        .unwrap();

        let resp = executor.exec(&mut backend, &genesis.txs, &[]);

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
        MetadataController::new(Arc::new(adapter), epoch_len, METADATA_CONTRACT_ADDRESS)
    }

    pub fn exec(&mut self, txs: Vec<SignedTransaction>) {
        let mut backend = AxonExecutorAdapter::from_root(
            self.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            mock_proposal().into(),
        )
        .unwrap();
        let resp = AxonExecutor::default().exec(&mut backend, &txs, &[]);
        self.state_root = resp.state_root;
    }
}

fn mock_header(block_number: u64, state: H256) -> Header {
    Header {
        prev_hash:         Default::default(),
        proposer:          Default::default(),
        transactions_root: Default::default(),
        signed_txs_hash:   Default::default(),
        timestamp:         Default::default(),
        number:            block_number,
        gas_limit:         1000000000u64.into(),
        extra_data:        Default::default(),
        mixed_hash:        Default::default(),
        base_fee_per_gas:  U256::one(),
        proof:             Default::default(),
        chain_id:          Default::default(),
        receipts_root:     Default::default(),
        log_bloom:         Default::default(),
        difficulty:        U256::one(),
        nonce:             Default::default(),
        gas_used:          U256::one(),
        state_root:        state,
    }
}

fn mock_proposal() -> Proposal {
    Proposal {
        prev_hash:         Default::default(),
        proposer:          Default::default(),
        transactions_root: Default::default(),
        signed_txs_hash:   Default::default(),
        timestamp:         Default::default(),
        number:            Default::default(),
        gas_limit:         1000000000u64.into(),
        extra_data:        Default::default(),
        base_fee_per_gas:  U256::one(),
        proof:             Default::default(),
        chain_id:          Default::default(),
        tx_hashes:         vec![],
    }
}

fn mock_transaction(nonce: u64, data: Vec<u8>) -> Eip1559Transaction {
    Eip1559Transaction {
        nonce:                    nonce.into(),
        gas_limit:                100000000u64.into(),
        max_priority_fee_per_gas: U256::one(),
        gas_price:                U256::one(),
        action:                   TransactionAction::Call(METADATA_CONTRACT_ADDRESS),
        value:                    U256::zero(),
        data:                     data.into(),
        access_list:              vec![],
    }
}

fn mock_signed_tx(nonce: u64, data: Vec<u8>) -> SignedTransaction {
    let raw = mock_transaction(nonce, data);
    let tx = UnverifiedTransaction {
        unsigned:  UnsignedTransaction::Eip1559(raw),
        signature: Some(SignatureComponents {
            standard_v: 2,
            r:          Bytes::default(),
            s:          Bytes::default(),
        }),
        chain_id:  0,
        hash:      Default::default(),
    };

    SignedTransaction {
        transaction: tx.calc_hash(),
        sender:      Address::from_hex("0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1")
            .unwrap()
            .0,
        public:      Some(Public::default()),
    }
}

fn mock_metadata(epoch: u64, start: u64, end: u64) -> Vec<u8> {
    let r = BufReader::new(File::open("./src/tests/metadata.json").unwrap());
    let mut metadata: Metadata = serde_json::from_reader(r).unwrap();
    metadata.epoch = epoch;
    metadata.version.start = start;
    metadata.version.end = end;
    let call = abi::MetadataContractCalls::AppendMetadata(abi::AppendMetadataCall {
        metadata: metadata.into(),
    });
    call.encode()
}
