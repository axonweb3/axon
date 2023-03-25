mod verify_in_mempool;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use core_executor::adapter::{MPTTrie, RocksTrieDB};
use core_executor::{system_contract::system_contract_address, AxonExecutor, AxonExecutorAdapter};
use core_rpc_client::RpcClient;
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use protocol::codec::ProtocolCodec;
use protocol::traits::{CommonStorage, Context, Executor, Storage};
use protocol::types::{
    Account, Address, Bytes, Eip1559Transaction, ExecResp, Proposal, Public, RichBlock,
    SignatureComponents, SignedTransaction, TransactionAction, UnsignedTransaction,
    UnverifiedTransaction, H256, NIL_DATA, RLP_NULL, U256,
};

const GENESIS_PATH: &str = "../../devtools/chain/genesis_single_node.json";

lazy_static::lazy_static! {
    pub static ref RPC: RpcClient = init_rpc_client();
}

struct TestHandle {
    storage:    Arc<ImplStorage<RocksAdapter>>,
    trie_db:    Arc<RocksTrieDB>,
    state_root: H256,
}

impl TestHandle {
    pub async fn new(salt: u64) -> Self {
        let path = "./free-space/interoperation/".to_string();
        let storage_adapter = RocksAdapter::new(
            path.clone() + &salt.to_string() + "/rocks",
            Default::default(),
        )
        .unwrap();
        let trie_db = RocksTrieDB::new(
            path.clone() + &salt.to_string() + "/trie",
            Default::default(),
            50,
        )
        .unwrap();

        let mut handle = TestHandle {
            storage:    Arc::new(ImplStorage::new(Arc::new(storage_adapter), 10)),
            trie_db:    Arc::new(trie_db),
            state_root: H256::default(),
        };
        handle.load_genesis().await;

        let mut backend = AxonExecutorAdapter::from_root(
            handle.state_root,
            Arc::clone(&handle.trie_db),
            Arc::clone(&handle.storage),
            mock_proposal().into(),
        )
        .unwrap();

        core_executor::system_contract::init(
            path + &salt.to_string() + "/sc",
            Default::default(),
            &mut backend,
        );

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

    pub fn exec(&mut self, txs: Vec<SignedTransaction>) -> ExecResp {
        let mut backend = AxonExecutorAdapter::from_root(
            self.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            mock_proposal().into(),
        )
        .unwrap();
        let resp = AxonExecutor::default().exec(&mut backend, &txs, &[]);
        self.state_root = resp.state_root;
        resp
    }
}

fn mock_proposal() -> Proposal {
    Proposal {
        prev_hash:                  Default::default(),
        proposer:                   Default::default(),
        transactions_root:          Default::default(),
        signed_txs_hash:            Default::default(),
        timestamp:                  Default::default(),
        number:                     Default::default(),
        gas_limit:                  1000000000u64.into(),
        extra_data:                 Default::default(),
        mixed_hash:                 Default::default(),
        base_fee_per_gas:           U256::one(),
        proof:                      Default::default(),
        last_checkpoint_block_hash: Default::default(),
        chain_id:                   Default::default(),
        call_system_script_count:   1,
        tx_hashes:                  vec![],
    }
}

fn mock_transaction(nonce: u64, data: Vec<u8>) -> Eip1559Transaction {
    Eip1559Transaction {
        nonce:                    nonce.into(),
        gas_limit:                100000000u64.into(),
        max_priority_fee_per_gas: U256::one(),
        gas_price:                U256::one(),
        action:                   TransactionAction::Call(system_contract_address(0x1)),
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

fn init_rpc_client() -> RpcClient {
    RpcClient::new(
        "https://testnet.ckb.dev/",
        "http://127.0.0.1:8116",
        "http://127.0.0.1:8118",
    )
}
