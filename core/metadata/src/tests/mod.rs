mod controller;

use std::fs::{self, File};
use std::io::BufReader;
use std::sync::Arc;

use ethers::core::abi::AbiEncode;

use core_executor::adapter::{MPTTrie, RocksTrieDB};
use core_executor::{EVMExecutorAdapter, EvmExecutor};
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use protocol::codec::ProtocolCodec;
use protocol::traits::{CommonStorage, Context, Executor, Storage};
use protocol::types::{
    Account, Address, Header, Hex, Metadata, MetadataVersion, Proposal, Public, RichBlock,
    SignatureComponents, SignedTransaction, Transaction, TransactionAction, UnverifiedTransaction,
    ValidatorExtend, H160, H256, NIL_DATA, RLP_NULL, U256,
};

use crate::{calc_epoch, metadata_abi as abi, MetadataAdapterImpl, MetadataController, EPOCH_LEN};

const GENESIS_PATH: &str = "../../devtools/chain/genesis_single_node.json";

lazy_static::lazy_static! {
    static ref METADATA_ADDRESS: H160
        = H160::from_slice(&Hex::decode("0x4af5ec5e3d29d9ddd7f4bf91a022131c41b72352".to_string()).unwrap());
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
    pub async fn new(salt: u64) -> Self {
        let path = "./free-space/".to_string();
        let storage_adapter =
            RocksAdapter::new(path.clone() + &salt.to_string() + "/rocks", 1024).unwrap();
        let trie_db = RocksTrieDB::new(path + &salt.to_string() + "/trie", 1024, 50).unwrap();

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
        MetadataController::new(Arc::new(adapter), *METADATA_ADDRESS, epoch_len)
    }

    pub fn exec(&mut self, txs: Vec<SignedTransaction>) {
        let mut backend = EVMExecutorAdapter::from_root(
            self.state_root,
            Arc::clone(&self.trie_db),
            Arc::clone(&self.storage),
            mock_proposal().into(),
        )
        .unwrap();
        let resp = EvmExecutor::default().exec(&mut backend, txs);
        println!("{:?}", resp);
        self.state_root = resp.state_root;
    }
}

fn mock_header(blocl_number: u64, state: H256) -> Header {
    Header {
        prev_hash:                  Default::default(),
        proposer:                   Default::default(),
        transactions_root:          Default::default(),
        signed_txs_hash:            Default::default(),
        timestamp:                  Default::default(),
        number:                     blocl_number,
        gas_limit:                  1000000000u64.into(),
        extra_data:                 Default::default(),
        mixed_hash:                 Default::default(),
        base_fee_per_gas:           U256::one(),
        proof:                      Default::default(),
        last_checkpoint_block_hash: Default::default(),
        chain_id:                   Default::default(),
        receipts_root:              Default::default(),
        log_bloom:                  Default::default(),
        difficulty:                 U256::one(),
        nonce:                      Default::default(),
        gas_used:                   U256::one(),
        state_root:                 state,
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
        tx_hashes:                  vec![],
    }
}

fn mock_transaction(nonce: u64, data: Vec<u8>) -> Transaction {
    Transaction {
        nonce:                    nonce.into(),
        gas_limit:                100000000u64.into(),
        max_priority_fee_per_gas: U256::one(),
        gas_price:                U256::one(),
        action:                   TransactionAction::Call(*METADATA_ADDRESS),
        value:                    U256::zero(),
        data:                     data.into(),
        access_list:              vec![],
    }
}

fn mock_signed_tx(nonce: u64, data: Vec<u8>) -> SignedTransaction {
    let raw = mock_transaction(nonce, data);
    let tx = UnverifiedTransaction {
        unsigned:  raw,
        signature: Some(SignatureComponents {
            standard_v: 0,
            r:          H256::default(),
            s:          H256::default(),
        }),
        chain_id:  0,
        hash:      Default::default(),
    };

    SignedTransaction {
        transaction: tx.hash(),
        sender:      Address::from_hex("0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1")
            .unwrap()
            .0,
        public:      Some(Public::default()),
    }
}

fn mock_metadata(epoch: u64, start: u64, end: u64) -> Vec<u8> {
    let r = BufReader::new(File::open("../../devtools/chain/metadata.json").unwrap());
    let mut metadata: Metadata = serde_json::from_reader(r).unwrap();
    metadata.epoch = epoch;
    metadata.version.start = start;
    metadata.version.end = end;
    let call = abi::MetadataContractCalls::AppendMetadata(abi::AppendMetadataCall {
        metadata: metadata.into(),
    });
    call.encode()
}

impl From<MetadataVersion> for abi::MetadataVersion {
    fn from(version: MetadataVersion) -> Self {
        abi::MetadataVersion {
            start: version.start,
            end:   version.end,
        }
    }
}

impl From<ValidatorExtend> for abi::ValidatorExtend {
    fn from(ve: ValidatorExtend) -> Self {
        abi::ValidatorExtend {
            bls_pub_key:    ve.bls_pub_key.as_bytes().into(),
            pub_key:        ve.pub_key.as_bytes().into(),
            address:        ve.address,
            propose_weight: ve.propose_weight,
            vote_weight:    ve.vote_weight,
        }
    }
}

impl From<Metadata> for abi::Metadata {
    fn from(m: Metadata) -> Self {
        abi::Metadata {
            version:                    m.version.into(),
            epoch:                      m.epoch,
            gas_limit:                  m.gas_limit,
            gas_price:                  m.gas_price,
            interval:                   m.interval,
            verifier_list:              m.verifier_list.into_iter().map(Into::into).collect(),
            propose_ratio:              m.propose_ratio,
            prevote_ratio:              m.prevote_ratio,
            precommit_ratio:            m.precommit_ratio,
            brake_ratio:                m.brake_ratio,
            tx_num_limit:               m.tx_num_limit,
            max_tx_size:                m.max_tx_size,
            last_checkpoint_block_hash: m.last_checkpoint_block_hash.0,
        }
    }
}
