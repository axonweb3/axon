#![allow(clippy::uninlined_format_args, clippy::mutable_key_type)]

use std::{collections::HashMap, panic::PanicInfo, path::Path, sync::Arc, time::Duration};

use backtrace::Backtrace;
#[cfg(all(
    not(target_env = "msvc"),
    not(target_os = "macos"),
    feature = "jemalloc"
))]
use {
    jemalloc_ctl::{Access, AsName},
    jemallocator::Jemalloc,
};

use common_apm::metrics::mempool::{MEMPOOL_CO_QUEUE_LEN, MEMPOOL_LEN_GAUGE};
use common_apm::{server::run_prometheus_server, tracing::global_tracer_register};
use common_config_parser::types::spec::{ChainSpec, InitialAccount};
use common_config_parser::types::{Config, ConfigJaeger, ConfigPrometheus, ConfigRocksDB};
use common_crypto::{BlsPrivateKey, BlsPublicKey, Secp256k1, Secp256k1PrivateKey, ToPublicKey};

use protocol::codec::{hex_decode, ProtocolCodec};
#[cfg(unix)]
use protocol::tokio::signal::unix as os_impl;
use protocol::tokio::{
    self, runtime::Builder as RuntimeBuilder, sync::Mutex as AsyncMutex, time::sleep,
};
use protocol::traits::{
    Consensus, Context, Executor, Gossip, MemPool, Network, NodeInfo, PeerTrust, ReadOnlyStorage,
    Rpc, Storage, SynchronizationAdapter,
};
use protocol::types::{
    Account, Block, ExecResp, MerkleRoot, Metadata, Proposal, RichBlock, SignedTransaction,
    Validator, ValidatorExtend, H256, NIL_DATA, RLP_NULL,
};
use protocol::{
    async_trait, lazy::CHAIN_ID, trie::DB as TrieDB, Display, From, ProtocolError,
    ProtocolErrorKind, ProtocolResult,
};

use core_api::{jsonrpc::run_jsonrpc_server, DefaultAPIAdapter};
use core_consensus::message::{
    ChokeMessageHandler, ProposalMessageHandler, PullBlockRpcHandler, PullProofRpcHandler,
    PullTxsRpcHandler, QCMessageHandler, RemoteHeightMessageHandler, VoteMessageHandler,
    BROADCAST_HEIGHT, END_GOSSIP_AGGREGATED_VOTE, END_GOSSIP_SIGNED_CHOKE,
    END_GOSSIP_SIGNED_PROPOSAL, END_GOSSIP_SIGNED_VOTE, RPC_RESP_SYNC_PULL_BLOCK,
    RPC_RESP_SYNC_PULL_PROOF, RPC_RESP_SYNC_PULL_TXS, RPC_SYNC_PULL_BLOCK, RPC_SYNC_PULL_PROOF,
    RPC_SYNC_PULL_TXS,
};
use core_consensus::status::{CurrentStatus, StatusAgent};
use core_consensus::{
    util::OverlordCrypto, ConsensusWal, DurationConfig, OverlordConsensus,
    OverlordConsensusAdapter, OverlordSynchronization, SignedTxsWAL,
};
use core_db::{RocksAdapter, RocksDB};
use core_executor::system_contract::{self, metadata::MetadataHandle};
use core_executor::{
    AxonExecutor, AxonExecutorApplyAdapter, AxonExecutorReadOnlyAdapter, MPTTrie, RocksTrieDB,
};
use core_interoperation::InteroperationImpl;
use core_mempool::{
    DefaultMemPoolAdapter, MemPoolImpl, NewTxsHandler, PullTxsHandler, END_GOSSIP_NEW_TXS,
    RPC_PULL_TXS, RPC_RESP_PULL_TXS, RPC_RESP_PULL_TXS_SYNC,
};
use core_network::{
    observe_listen_port_occupancy, NetworkConfig, NetworkService, PeerId, PeerIdExt, SecioError,
};
use core_storage::ImplStorage;

pub use core_network::{KeyProvider, SecioKeyPair};

#[cfg(all(
    not(target_env = "msvc"),
    not(target_os = "macos"),
    feature = "jemalloc"
))]
#[global_allocator]
pub static JEMALLOC: Jemalloc = Jemalloc;

#[derive(Debug)]
pub struct Axon {
    version:    String,
    config:     Config,
    spec:       ChainSpec,
    genesis:    RichBlock,
    state_root: MerkleRoot,
}

#[cfg(test)]
mod tests;

impl Axon {
    pub fn new(version: String, config: Config, spec: ChainSpec, genesis: RichBlock) -> Axon {
        Axon {
            version,
            config,
            spec,
            genesis,
            state_root: MerkleRoot::default(),
        }
    }

    pub fn run<K: KeyProvider>(mut self, key_provider: Option<K>) -> ProtocolResult<()> {
        #[cfg(all(
            not(target_env = "msvc"),
            not(target_os = "macos"),
            feature = "jemalloc"
        ))]
        Self::set_profile(true);

        let rt = RuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .expect("new tokio runtime");

        rt.block_on(async move {
            let (storage, trie_db, inner_db) = init_storage(
                &self.config.rocksdb,
                self.config.data_path_for_rocksdb(),
                self.config.executor.triedb_cache_size,
            )
            .await?;
            if let Some(genesis) = self.try_load_genesis(&storage).await? {
                log::info!("The Genesis block has been initialized.");
                self.apply_genesis_after_checks(&genesis).await?;
            } else {
                self.create_genesis(&storage, &trie_db, &inner_db).await?;
            }

            self.start(key_provider, storage, trie_db, inner_db).await
        })?;
        rt.shutdown_timeout(std::time::Duration::from_secs(1));

        Ok(())
    }

    async fn try_load_genesis(
        &self,
        storage: &Arc<ImplStorage<RocksAdapter>>,
    ) -> ProtocolResult<Option<Block>> {
        storage.get_block(Context::new(), 0).await.or_else(|e| {
            if e.to_string().contains("GetNone") {
                Ok(None)
            } else {
                Err(e)
            }
        })
    }

    async fn create_genesis(
        &mut self,
        storage: &Arc<ImplStorage<RocksAdapter>>,
        trie_db: &Arc<RocksTrieDB>,
        inner_db: &Arc<RocksDB>,
    ) -> ProtocolResult<()> {
        let resp = execute_transactions(
            &self.genesis,
            storage,
            trie_db,
            inner_db,
            &self.spec.accounts,
        )?;

        log::info!(
            "Execute the genesis distribute success, genesis state root {:?}, response {:?}",
            resp.state_root,
            resp
        );

        self.state_root = resp.state_root;
        self.apply_genesis_data(resp.state_root, resp.receipt_root)?;

        log::info!("The genesis block is created {:?}", self.genesis.block);

        save_block(storage, &self.genesis, &resp).await?;

        Ok(())
    }

    async fn apply_genesis_after_checks(&mut self, loaded_genesis: &Block) -> ProtocolResult<()> {
        let tmp_dir = tempfile::tempdir().map_err(|err| {
            let err_msg = format!("failed to create temporary directory since {err:?}");
            MainError::Other(err_msg)
        })?;
        let path_block = tmp_dir.path().join("block");

        let (storage, trie_db, inner_db) = init_storage(
            &self.config.rocksdb,
            path_block,
            self.config.executor.triedb_cache_size,
        )
        .await?;

        let resp = execute_transactions(
            &self.genesis,
            &storage,
            &trie_db,
            &inner_db,
            &self.spec.accounts,
        )?;

        self.apply_genesis_data(resp.state_root, resp.receipt_root)?;

        let user_provided_genesis = &self.genesis.block;
        if user_provided_genesis != loaded_genesis {
            let err_msg = format!(
                "The user provided genesis (hash: {:#x}) is NOT \
                the same as the genesis in storage (hash: {:#x})",
                user_provided_genesis.hash(),
                loaded_genesis.hash()
            );
            return Err(MainError::Other(err_msg).into());
        }

        Ok(())
    }

    fn apply_genesis_data(&mut self, state_root: H256, receipts_root: H256) -> ProtocolResult<()> {
        if self.genesis.block.header.state_root.is_zero() {
            self.genesis.block.header.state_root = state_root;
        } else if self.genesis.block.header.state_root != state_root {
            let errmsg = format!(
                "The state root of genesis block which user provided is incorrect, \
                if you don't know it, you can just set it as {:#x}.",
                H256::default()
            );
            return Err(MainError::Other(errmsg).into());
        }
        if self.genesis.block.header.receipts_root.is_zero() {
            self.genesis.block.header.receipts_root = receipts_root;
        } else if self.genesis.block.header.receipts_root != receipts_root {
            let errmsg = format!(
                "The receipts root of genesis block which user provided is incorrect, \
                if you don't know it, you can just set it as {:#x}.",
                H256::default()
            );
            return Err(MainError::Other(errmsg).into());
        }
        Ok(())
    }

    pub async fn start<K: KeyProvider>(
        self,
        key_provider: Option<K>,
        storage: Arc<ImplStorage<RocksAdapter>>,
        trie_db: Arc<RocksTrieDB>,
        inner_db: Arc<RocksDB>,
    ) -> ProtocolResult<()> {
        // Start jaeger
        Self::run_jaeger(self.config.jaeger.clone());

        // Start prometheus http server
        Self::run_prometheus_server(self.config.prometheus.clone());

        #[cfg(all(
            not(target_env = "msvc"),
            not(target_os = "macos"),
            feature = "jemalloc"
        ))]
        tokio::spawn(common_memory_tracker::track_db_process(
            "blockdb",
            Arc::clone(&inner_db),
        ));

        #[cfg(all(
            not(target_env = "msvc"),
            not(target_os = "macos"),
            feature = "jemalloc"
        ))]
        tokio::spawn(common_memory_tracker::track_current_process());

        log::info!("node starts");

        observe_listen_port_occupancy(&[self.config.network.listening_address.clone()]).await?;

        // Init Block db and get the current block
        let current_block = storage.get_latest_block(Context::new()).await?;
        let current_state_root = current_block.header.state_root;

        log::info!("At block number {}", current_block.header.number + 1);

        // Init network
        let mut network_service = self.init_network_service(key_provider);

        // Init full transactions wal
        let txs_wal_path = self
            .config
            .data_path_for_txs_wal()
            .to_str()
            .unwrap()
            .to_string();
        let txs_wal = Arc::new(SignedTxsWAL::new(txs_wal_path));

        // Init system contract
        if current_block.header.number != 0 {
            let mut backend = AxonExecutorApplyAdapter::from_root(
                current_block.header.state_root,
                Arc::clone(&trie_db),
                Arc::clone(&storage),
                Proposal::new_without_state_root(&current_block.header).into(),
            )
            .unwrap();

            system_contract::init(inner_db, &mut backend);
        }

        // Init mempool and recover signed transactions with the current block number
        let current_stxs = txs_wal.load_by_number(current_block.header.number + 1);
        log::info!("Recover {} txs from wal", current_stxs.len());

        let mempool = self
            .init_mempool(&trie_db, &network_service.handle(), &storage, &current_stxs)
            .await;

        // Get the validator list from current metadata for consensus initialization
        let metadata_root = AxonExecutorReadOnlyAdapter::from_root(
            current_state_root,
            Arc::clone(&trie_db),
            Arc::clone(&storage),
            Proposal::new_without_state_root(&self.genesis.block.header).into(),
        )?
        .get_metadata_root();
        let metadata = MetadataHandle::new(metadata_root)
            .get_metadata_by_block_number(current_block.header.number)?;
        let validators: Vec<Validator> = metadata
            .verifier_list
            .iter()
            .map(|v| Validator {
                pub_key:        v.pub_key.as_bytes(),
                propose_weight: v.propose_weight,
                vote_weight:    v.vote_weight,
            })
            .collect::<Vec<_>>();

        // Set args in mempool
        mempool.set_args(
            Context::new(),
            current_block.header.state_root,
            metadata.consensus_config.gas_limit,
            metadata.consensus_config.max_tx_size,
        );

        // Init overlord consensus and synchronization
        let lock = Arc::new(AsyncMutex::new(()));
        let crypto = self.init_crypto(&metadata.verifier_list);
        let consensus_adapter = OverlordConsensusAdapter::<_, _, _, _>::new(
            Arc::new(network_service.handle()),
            Arc::clone(&mempool),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
            Arc::clone(&crypto),
        )?;
        let consensus_adapter = Arc::new(consensus_adapter);
        let status_agent = self
            .get_status_agent(&storage, &current_block, &metadata)
            .await;

        let overlord_consensus = self
            .init_overlord_consensus(&status_agent, &txs_wal, &crypto, &lock, &consensus_adapter)
            .await;

        consensus_adapter.set_overlord_handler(overlord_consensus.get_overlord_handler());

        let synchronization = Arc::new(OverlordSynchronization::<_>::new(
            self.config.sync.sync_txs_chunk_size,
            consensus_adapter,
            status_agent.clone(),
            lock,
        ));

        self.tag_consensus(&network_service, &metadata.verifier_list);

        // register endpoints to network service
        self.register_mempool_endpoint(&mut network_service, &mempool);
        self.register_consensus_endpoint(&mut network_service, &overlord_consensus);
        self.register_synchronization_endpoint(&mut network_service, &synchronization);
        self.register_storage_endpoint(&mut network_service, &storage);
        self.register_rpc(&mut network_service);

        let network_handle = network_service.handle();

        // Run network service at the end of its life cycle
        tokio::spawn(network_service.run());

        // Run API
        let api_adapter = Arc::new(DefaultAPIAdapter::new(
            Arc::clone(&mempool),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
            Arc::new(network_handle),
        ));
        let _handles = run_jsonrpc_server(self.version, self.config.clone(), api_adapter).await?;

        // Run sync
        tokio::spawn(async move {
            if let Err(e) = synchronization.polling_broadcast().await {
                log::error!("synchronization: {:?}", e);
            }
        });

        // Run consensus
        Self::run_overlord_consensus(metadata, validators, current_block, overlord_consensus);

        Self::set_ctrl_c_handle().await;

        Ok(())
    }

    fn init_network_service<K: KeyProvider>(
        &self,
        key_provider: Option<K>,
    ) -> NetworkService<KeyP<K>> {
        let network_config =
            NetworkConfig::from_config(&self.config, self.genesis.block.header.chain_id).unwrap();

        let key = key_provider
            .map(KeyP::Custom)
            .unwrap_or(KeyP::Default(network_config.secio_keypair.clone()));

        NetworkService::new(network_config, key)
    }

    async fn init_mempool<N, S, DB>(
        &self,
        trie_db: &Arc<DB>,
        network_service: &N,
        storage: &Arc<S>,
        signed_txs: &[SignedTransaction],
    ) -> Arc<MemPoolImpl<DefaultMemPoolAdapter<Secp256k1, N, S, DB, InteroperationImpl>>>
    where
        N: Rpc + PeerTrust + Gossip + Clone + Unpin + 'static,
        S: Storage + 'static,
        DB: TrieDB + Send + Sync + 'static,
    {
        let mempool_adapter = DefaultMemPoolAdapter::<Secp256k1, _, _, _, InteroperationImpl>::new(
            network_service.clone(),
            Arc::clone(storage),
            Arc::clone(trie_db),
            self.genesis.block.header.chain_id,
            self.genesis.block.header.gas_limit.as_u64(),
            self.config.mempool.pool_size as usize,
            self.config.mempool.broadcast_txs_size,
            self.config.mempool.broadcast_txs_interval,
        );
        let mempool = Arc::new(
            MemPoolImpl::new(
                self.config.mempool.pool_size as usize,
                self.config.mempool.timeout_gap,
                mempool_adapter,
                signed_txs.to_owned(),
            )
            .await,
        );

        // Clone the mempool and spawn a thread to monitor the mempool length.
        let monitor_mempool = Arc::clone(&mempool);
        tokio::spawn(async move {
            let interval = Duration::from_millis(1000);
            loop {
                sleep(interval).await;
                MEMPOOL_LEN_GAUGE.set(monitor_mempool.len() as i64);
                MEMPOOL_CO_QUEUE_LEN.set(monitor_mempool.len() as i64);
            }
        });

        mempool
    }

    fn init_crypto(&self, validators: &[ValidatorExtend]) -> Arc<OverlordCrypto> {
        // self private key
        let bls_priv_key = BlsPrivateKey::try_from(self.config.privkey.as_ref())
            .map_err(MainError::Crypto)
            .unwrap();

        let mut bls_pub_keys = HashMap::new();
        for validator_extend in validators.iter() {
            let address = validator_extend.pub_key.as_bytes();
            let hex_pubkey = hex_decode(&validator_extend.bls_pub_key.as_string_trim0x()).unwrap();
            let pub_key = BlsPublicKey::try_from(hex_pubkey.as_ref())
                .map_err(MainError::Crypto)
                .unwrap();
            bls_pub_keys.insert(address, pub_key);
        }

        Arc::new(OverlordCrypto::new(
            bls_priv_key,
            bls_pub_keys,
            String::new(),
        ))
    }

    async fn get_status_agent(
        &self,
        storage: &Arc<impl Storage>,
        block: &Block,
        metadata: &Metadata,
    ) -> StatusAgent {
        let header = &block.header;
        let latest_proof = storage.get_latest_proof(Context::new()).await.unwrap();
        let current_consensus_status = CurrentStatus {
            prev_hash:       block.hash(),
            last_number:     header.number,
            max_tx_size:     metadata.consensus_config.max_tx_size.into(),
            tx_num_limit:    metadata.consensus_config.tx_num_limit,
            proof:           latest_proof,
            last_state_root: if header.number == 0 {
                self.state_root
            } else {
                header.state_root
            },
        };

        CHAIN_ID.swap(Arc::new(header.chain_id));

        StatusAgent::new(current_consensus_status)
    }

    async fn init_overlord_consensus<M, N, S, DB>(
        &self,
        status_agent: &StatusAgent,
        txs_wal: &Arc<SignedTxsWAL>,
        crypto: &Arc<OverlordCrypto>,
        lock: &Arc<AsyncMutex<()>>,
        consensus_adapter: &Arc<OverlordConsensusAdapter<M, N, S, DB>>,
    ) -> Arc<OverlordConsensus<OverlordConsensusAdapter<M, N, S, DB>>>
    where
        M: MemPool + 'static,
        N: Rpc + PeerTrust + Gossip + Network + 'static,
        S: Storage + 'static,
        DB: TrieDB + Send + Sync + 'static,
    {
        let consensus_wal_path = self
            .config
            .data_path_for_consensus_wal()
            .to_str()
            .unwrap()
            .to_string();
        let consensus_wal = Arc::new(ConsensusWal::new(consensus_wal_path));

        let my_privkey = Secp256k1PrivateKey::try_from(self.config.privkey.as_ref())
            .map_err(MainError::Crypto)
            .unwrap();
        let node_info = NodeInfo::new(self.genesis.block.header.chain_id, my_privkey.pub_key());

        Arc::new(
            OverlordConsensus::new(
                status_agent.clone(),
                node_info,
                Arc::clone(crypto),
                Arc::clone(txs_wal),
                Arc::clone(consensus_adapter),
                Arc::clone(lock),
                Arc::clone(&consensus_wal),
            )
            .await,
        )
    }

    fn tag_consensus<K: KeyProvider>(
        &self,
        network_service: &NetworkService<K>,
        validators: &[ValidatorExtend],
    ) {
        let peer_ids = validators
            .iter()
            .map(|v| PeerId::from_pubkey_bytes(v.pub_key.as_bytes()).map(PeerIdExt::into_bytes_ext))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        network_service
            .handle()
            .tag_consensus(Context::new(), peer_ids)
            .unwrap();
    }

    fn register_mempool_endpoint<K: KeyProvider>(
        &self,
        network_service: &mut NetworkService<K>,
        mempool: &Arc<impl MemPool + 'static>,
    ) {
        // register broadcast new transaction
        network_service
            .register_endpoint_handler(END_GOSSIP_NEW_TXS, NewTxsHandler::new(Arc::clone(mempool)))
            .unwrap();

        // register pull txs from other node
        network_service
            .register_endpoint_handler(
                RPC_PULL_TXS,
                PullTxsHandler::new(Arc::new(network_service.handle()), Arc::clone(mempool)),
            )
            .unwrap();
    }

    fn register_consensus_endpoint<K: KeyProvider>(
        &self,
        network_service: &mut NetworkService<K>,
        overlord_consensus: &Arc<impl Consensus + 'static>,
    ) {
        // register consensus
        network_service
            .register_endpoint_handler(
                END_GOSSIP_SIGNED_PROPOSAL,
                ProposalMessageHandler::new(Arc::clone(overlord_consensus)),
            )
            .unwrap();
        network_service
            .register_endpoint_handler(
                END_GOSSIP_AGGREGATED_VOTE,
                QCMessageHandler::new(Arc::clone(overlord_consensus)),
            )
            .unwrap();
        network_service
            .register_endpoint_handler(
                END_GOSSIP_SIGNED_VOTE,
                VoteMessageHandler::new(Arc::clone(overlord_consensus)),
            )
            .unwrap();
        network_service
            .register_endpoint_handler(
                END_GOSSIP_SIGNED_CHOKE,
                ChokeMessageHandler::new(Arc::clone(overlord_consensus)),
            )
            .unwrap();
    }

    fn register_synchronization_endpoint<K: KeyProvider>(
        &self,
        network_service: &mut NetworkService<K>,
        synchronization: &Arc<OverlordSynchronization<impl SynchronizationAdapter + 'static>>,
    ) {
        network_service
            .register_endpoint_handler(
                BROADCAST_HEIGHT,
                RemoteHeightMessageHandler::new(Arc::clone(synchronization)),
            )
            .unwrap();
    }

    fn register_storage_endpoint<K: KeyProvider>(
        &self,
        network_service: &mut NetworkService<K>,
        storage: &Arc<ImplStorage<RocksAdapter>>,
    ) {
        // register storage
        network_service
            .register_endpoint_handler(
                RPC_SYNC_PULL_BLOCK,
                PullBlockRpcHandler::new(Arc::new(network_service.handle()), Arc::clone(storage)),
            )
            .unwrap();

        network_service
            .register_endpoint_handler(
                RPC_SYNC_PULL_PROOF,
                PullProofRpcHandler::new(Arc::new(network_service.handle()), Arc::clone(storage)),
            )
            .unwrap();

        network_service
            .register_endpoint_handler(
                RPC_SYNC_PULL_TXS,
                PullTxsRpcHandler::new(Arc::new(network_service.handle()), Arc::clone(storage)),
            )
            .unwrap();
    }

    fn register_rpc<K: KeyProvider>(&self, network_service: &mut NetworkService<K>) {
        network_service
            .register_rpc_response(RPC_RESP_PULL_TXS)
            .unwrap();
        network_service
            .register_rpc_response(RPC_RESP_PULL_TXS_SYNC)
            .unwrap();
        network_service
            .register_rpc_response(RPC_RESP_SYNC_PULL_BLOCK)
            .unwrap();
        network_service
            .register_rpc_response(RPC_RESP_SYNC_PULL_PROOF)
            .unwrap();
        network_service
            .register_rpc_response(RPC_RESP_SYNC_PULL_TXS)
            .unwrap();
    }

    fn run_jaeger(config: Option<ConfigJaeger>) {
        if let Some(jaeger_config) = config {
            let service_name = match jaeger_config.service_name {
                Some(name) => name,
                None => "axon".to_string(),
            };

            let tracing_address = match jaeger_config.tracing_address {
                Some(address) => address,
                None => std::net::SocketAddr::from(([0, 0, 0, 0], 6831)),
            };

            let tracing_batch_size = jaeger_config.tracing_batch_size.unwrap_or(50);

            global_tracer_register(&service_name, tracing_address, tracing_batch_size);
            log::info!("jaeger started");
        };
    }

    fn run_prometheus_server(config: Option<ConfigPrometheus>) {
        if let Some(prometheus_config) = config {
            if let Some(prometheus_listening_address) = prometheus_config.listening_address {
                tokio::spawn(run_prometheus_server(prometheus_listening_address));
                log::info!("prometheus started");
            }
        };
    }

    fn run_overlord_consensus<M, N, S, DB>(
        metadata: Metadata,
        validators: Vec<Validator>,
        current_block: Block,
        overlord_consensus: Arc<OverlordConsensus<OverlordConsensusAdapter<M, N, S, DB>>>,
    ) where
        M: MemPool,
        N: Rpc + PeerTrust + Gossip + Network + 'static,
        S: Storage,
        DB: TrieDB + Send + Sync,
    {
        let timer_config = DurationConfig {
            propose_ratio:   metadata.consensus_config.propose_ratio,
            prevote_ratio:   metadata.consensus_config.prevote_ratio,
            precommit_ratio: metadata.consensus_config.precommit_ratio,
            brake_ratio:     metadata.consensus_config.brake_ratio,
        };

        tokio::spawn(async move {
            if let Err(e) = overlord_consensus
                .run(
                    current_block.header.number,
                    metadata.consensus_config.interval,
                    validators,
                    Some(timer_config),
                )
                .await
            {
                log::error!("axon-consensus: {:?} error", e);
            }
        });
    }

    async fn set_ctrl_c_handle() {
        let ctrl_c_handler = tokio::spawn(async {
            #[cfg(windows)]
            let _ = tokio::signal::ctrl_c().await;
            #[cfg(unix)]
            {
                let mut sigtun_int = os_impl::signal(os_impl::SignalKind::interrupt()).unwrap();
                let mut sigtun_term = os_impl::signal(os_impl::SignalKind::terminate()).unwrap();
                tokio::select! {
                    _ = sigtun_int.recv() => {}
                    _ = sigtun_term.recv() => {}
                };
            }
        });

        // register channel of panic
        let (panic_sender, mut panic_receiver) = tokio::sync::mpsc::channel::<()>(1);

        std::panic::set_hook(Box::new(move |info: &PanicInfo| {
            let panic_sender = panic_sender.clone();
            Self::panic_log(info);
            panic_sender.try_send(()).expect("panic_receiver is droped");
        }));

        tokio::select! {
            _ = ctrl_c_handler => { log::info!("ctrl + c is pressed, quit.") },
            _ = panic_receiver.recv() => { log::info!("child thread panic, quit.") },
        };

        #[cfg(all(
            not(target_env = "msvc"),
            not(target_os = "macos"),
            feature = "jemalloc"
        ))]
        {
            Self::set_profile(false);
            Self::dump_profile();
        }
    }

    fn panic_log(info: &PanicInfo) {
        let backtrace = Backtrace::new();
        let thread = std::thread::current();
        let name = thread.name().unwrap_or("unnamed");
        let location = info.location().unwrap(); // The current implementation always returns Some
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };
        log::error!(
            target: "panic", "thread '{}' panicked at '{}': {}:{} {:?}",
            name,
            msg,
            location.file(),
            location.line(),
            backtrace,
        );
    }

    #[cfg(all(
        not(target_env = "msvc"),
        not(target_os = "macos"),
        feature = "jemalloc"
    ))]
    fn set_profile(is_active: bool) {
        let _ = b"prof.active\0"
            .name()
            .write(is_active)
            .map_err(|e| panic!("Set jemalloc profile error {:?}", e));
    }

    #[cfg(all(
        not(target_env = "msvc"),
        not(target_os = "macos"),
        feature = "jemalloc"
    ))]
    fn dump_profile() {
        let name = b"profile.out\0".as_ref();
        b"prof.dump\0"
            .name()
            .write(name)
            .expect("Should succeed to dump profile")
    }
}

#[derive(Debug, Display, From)]
pub enum MainError {
    #[display(fmt = "The axon configuration read failed {:?}", _0)]
    ConfigParse(common_config_parser::ParseError),

    #[display(fmt = "{:?}", _0)]
    Io(std::io::Error),

    #[display(fmt = "Toml fails to parse genesis {:?}", _0)]
    GenesisTomlDe(toml::de::Error),

    #[display(fmt = "crypto error {:?}", _0)]
    Crypto(common_crypto::Error),

    #[display(fmt = "{:?}", _0)]
    Utf8(std::string::FromUtf8Error),

    #[display(fmt = "{:?}", _0)]
    JSONParse(serde_json::error::Error),

    #[display(fmt = "other error {:?}", _0)]
    Other(String),
}

impl std::error::Error for MainError {}

impl From<MainError> for ProtocolError {
    fn from(error: MainError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Main, Box::new(error))
    }
}

#[derive(Clone)]
enum KeyP<K: KeyProvider> {
    Custom(K),
    Default(SecioKeyPair),
}
#[async_trait]
impl<K> KeyProvider for KeyP<K>
where
    K: KeyProvider,
{
    type Error = SecioError;

    async fn sign_ecdsa_async<T: AsRef<[u8]> + Send>(
        &self,
        message: T,
    ) -> Result<Vec<u8>, Self::Error> {
        match self {
            KeyP::Custom(k) => k.sign_ecdsa_async(message).await.map_err(Into::into),
            KeyP::Default(k) => k.sign_ecdsa_async(message).await,
        }
    }

    /// Constructs a signature for `msg` using the secret key `sk`
    fn sign_ecdsa<T: AsRef<[u8]>>(&self, message: T) -> Result<Vec<u8>, Self::Error> {
        match self {
            KeyP::Custom(k) => k.sign_ecdsa(message).map_err(Into::into),
            KeyP::Default(k) => k.sign_ecdsa(message),
        }
    }

    /// Creates a new public key from the [`KeyProvider`].
    fn pubkey(&self) -> Vec<u8> {
        match self {
            KeyP::Custom(k) => k.pubkey(),
            KeyP::Default(k) => k.pubkey(),
        }
    }

    /// Checks that `sig` is a valid ECDSA signature for `msg` using the
    /// pubkey.
    fn verify_ecdsa<P, T, F>(&self, pubkey: P, message: T, signature: F) -> bool
    where
        P: AsRef<[u8]>,
        T: AsRef<[u8]>,
        F: AsRef<[u8]>,
    {
        match self {
            KeyP::Custom(k) => k.verify_ecdsa(pubkey, message, signature),
            KeyP::Default(k) => k.verify_ecdsa(pubkey, message, signature),
        }
    }
}

async fn init_storage<P: AsRef<Path>>(
    config: &ConfigRocksDB,
    rocksdb_path: P,
    triedb_cache_size: usize,
) -> ProtocolResult<(
    Arc<ImplStorage<RocksAdapter>>,
    Arc<RocksTrieDB>,
    Arc<RocksDB>,
)> {
    let adapter = Arc::new(RocksAdapter::new(rocksdb_path, config.clone())?);
    let inner_db = adapter.inner_db();
    let trie_db = Arc::new(RocksTrieDB::new_evm(adapter.inner_db(), triedb_cache_size));
    let storage = Arc::new(ImplStorage::new(adapter, config.cache_size));
    Ok((storage, trie_db, inner_db))
}

fn insert_accounts(
    mpt: &mut MPTTrie<RocksTrieDB>,
    accounts: &[InitialAccount],
) -> ProtocolResult<()> {
    for account in accounts {
        let raw_account = Account {
            nonce:        0u64.into(),
            balance:      account.balance,
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        }
        .encode()?;
        mpt.insert(account.address.as_bytes(), &raw_account)?;
    }
    Ok(())
}

fn execute_transactions<S>(
    rich: &RichBlock,
    storage: &Arc<S>,
    trie_db: &Arc<RocksTrieDB>,
    inner_db: &Arc<RocksDB>,
    accounts: &[InitialAccount],
) -> ProtocolResult<ExecResp>
where
    S: Storage + 'static,
{
    let state_root = {
        let mut mpt = MPTTrie::new(Arc::clone(trie_db));
        insert_accounts(&mut mpt, accounts).expect("insert accounts");
        mpt.commit()?
    };
    let executor = AxonExecutor::default();
    let mut backend = AxonExecutorApplyAdapter::from_root(
        state_root,
        Arc::clone(trie_db),
        Arc::clone(storage),
        Proposal::new_without_state_root(&rich.block.header).into(),
    )?;

    system_contract::init(Arc::clone(inner_db), &mut backend);

    let resp = executor.exec(&mut backend, &rich.txs, &[]);

    resp.tx_resp.iter().enumerate().for_each(|(i, r)| {
        if !r.exit_reason.is_succeed() {
            panic!(
                "The {}th tx in genesis execute failed, reason {:?}",
                i, r.exit_reason
            );
        }
    });

    Ok(resp)
}

async fn save_block<S>(storage: &Arc<S>, rich: &RichBlock, resp: &ExecResp) -> ProtocolResult<()>
where
    S: Storage + 'static,
{
    storage
        .update_latest_proof(Context::new(), rich.block.header.proof.clone())
        .await?;
    storage
        .insert_block(Context::new(), rich.block.clone())
        .await?;
    storage
        .insert_transactions(Context::new(), rich.block.header.number, rich.txs.clone())
        .await?;
    let (receipts, _logs) = rich.generate_receipts_and_logs(resp);
    storage
        .insert_receipts(Context::new(), rich.block.header.number, receipts)
        .await?;
    Ok(())
}
