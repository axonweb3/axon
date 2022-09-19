#![allow(clippy::mutable_key_type)]

use std::{collections::HashMap, panic::PanicInfo, str::FromStr, sync::Arc, time::Duration};

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

use ethers_signers::{coins_bip39::English, MnemonicBuilder, Signer};

use common_apm::metrics::mempool::{MEMPOOL_CO_QUEUE_LEN, MEMPOOL_LEN_GAUGE};
use common_apm::{server::run_prometheus_server, tracing::global_tracer_register};
use common_config_parser::types::Config;
use common_crypto::{
    BlsPrivateKey, BlsPublicKey, PublicKey, Secp256k1, Secp256k1PrivateKey, ToPublicKey,
    UncompressedPublicKey,
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
    util::OverlordCrypto, ConsensusWal, DurationConfig, Node, OverlordConsensus,
    OverlordConsensusAdapter, OverlordSynchronization, SignedTxsWAL,
};
use core_cross_client::{
    CrossChainDBImpl, CrossChainImpl, CrossChainMessageHandler, DefaultCrossChainAdapter,
    END_GOSSIP_BUILD_CKB_TX, END_GOSSIP_CKB_TX_SIGNATURE,
};
use core_executor::{AxonExecutor, AxonExecutorAdapter, MPTTrie, RocksTrieDB};
use core_interoperation::InteroperationImpl;
use core_mempool::{
    DefaultMemPoolAdapter, MemPoolImpl, NewTxsHandler, PullTxsHandler, END_GOSSIP_NEW_TXS,
    RPC_PULL_TXS, RPC_RESP_PULL_TXS, RPC_RESP_PULL_TXS_SYNC,
};
use core_metadata::{MetadataAdapterImpl, MetadataController};
use core_network::{
    observe_listen_port_occupancy, NetworkConfig, NetworkService, PeerId, PeerIdExt,
};
use core_rpc_client::RpcClient;
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use core_tx_assembler::{IndexerAdapter, TxAssemblerImpl};
use protocol::lazy::{CHAIN_ID, CURRENT_STATE_ROOT};
#[cfg(unix)]
use protocol::tokio::signal::unix as os_impl;
use protocol::tokio::{runtime::Builder as RuntimeBuilder, sync::Mutex as AsyncMutex, time::sleep};
use protocol::traits::{
    CommonStorage, Context, Executor, MemPool, MetadataControl, Network, NodeInfo, Storage,
};
use protocol::types::{
    Account, Address, MerkleRoot, Proposal, RichBlock, Validator, H256, NIL_DATA, RLP_NULL,
};
use protocol::{
    codec::{hex_decode, ProtocolCodec},
    types::H160,
};
use protocol::{tokio, Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult};

#[cfg(all(
    not(target_env = "msvc"),
    not(target_os = "macos"),
    feature = "jemalloc"
))]
#[global_allocator]
pub static JEMALLOC: Jemalloc = Jemalloc;

#[derive(Debug)]
pub struct Axon {
    config:     Config,
    genesis:    RichBlock,
    state_root: MerkleRoot,
}

impl Axon {
    pub fn new(config: Config, genesis: RichBlock) -> Axon {
        Axon {
            config,
            genesis,
            state_root: MerkleRoot::default(),
        }
    }

    pub fn run(mut self) -> ProtocolResult<()> {
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
            self.create_genesis().await?;
            self.start().await
        })?;
        rt.shutdown_timeout(std::time::Duration::from_secs(1));
        Ok(())
    }

    pub async fn create_genesis(&mut self) -> ProtocolResult<()> {
        // Init Block db
        let path_block = self.config.data_path_for_block();
        let rocks_adapter = Arc::new(RocksAdapter::new(path_block, self.config.rocksdb.clone())?);
        let storage = Arc::new(ImplStorage::new(
            rocks_adapter,
            self.config.rocksdb.cache_size,
        ));

        match storage.get_latest_block(Context::new()).await {
            Ok(_) => {
                log::info!("The Genesis block has been initialized.");
                return Ok(());
            }
            Err(e) => {
                if !e.to_string().contains("GetNone") {
                    return Err(e);
                }
            }
        };

        // Init trie db
        let path_state = self.config.data_path_for_state();
        let trie_db = Arc::new(RocksTrieDB::new(
            path_state,
            self.config.rocksdb.clone(),
            self.config.executor.triedb_cache_size,
        )?);
        let mut mpt = MPTTrie::new(Arc::clone(&trie_db));

        let distribute_account = Account {
            nonce:        0u64.into(),
            balance:      self.config.accounts.balance,
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        }
        .encode()?;

        let genesis_sender = H160::from_str("8ab0cf264df99d83525e9e11c7e4db01558ae1b1").unwrap();
        mpt.insert(genesis_sender.as_bytes(), &distribute_account)?;

        let mut builder =
            MnemonicBuilder::<English>::default().phrase(self.config.accounts.mnemonic.as_str());
        let init_index = self.config.accounts.initial_index.unwrap_or(0);
        for i in init_index..(init_index + self.config.accounts.count) {
            builder = match &self.config.accounts.path {
                Some(path) => builder
                    .derivation_path(&format!("{}{}", path, i))
                    .map_err(MainError::WalletError)?,
                None => builder.index(i).map_err(MainError::WalletError)?,
            };
            let wallet = builder
                .build()
                .map_err(MainError::WalletError)?
                .with_chain_id(self.genesis.block.header.chain_id);
            mpt.insert(wallet.address().as_bytes(), &distribute_account)?;
        }

        let proposal = Proposal::from(&self.genesis.block);
        let executor = AxonExecutor::default();
        let mut backend = AxonExecutorAdapter::from_root(
            mpt.commit()?,
            trie_db,
            Arc::clone(&storage),
            proposal.into(),
        )?;
        let resp = executor.exec(&mut backend, &self.genesis.txs);

        self.state_root = resp.state_root;
        self.genesis.block.header.state_root = self.state_root;

        log::info!(
            "Execute the genesis distribute success, genesis state root {:?}, response {:?}",
            self.state_root,
            resp
        );

        storage
            .update_latest_proof(Context::new(), self.genesis.block.header.proof.clone())
            .await?;
        storage
            .insert_block(Context::new(), self.genesis.block.clone())
            .await?;
        storage
            .insert_transactions(
                Context::new(),
                self.genesis.block.header.number,
                self.genesis.txs.clone(),
            )
            .await?;

        log::info!("The genesis block is created {:?}", self.genesis.block);

        Ok(())
    }

    pub async fn start(self) -> ProtocolResult<()> {
        // Start jaeger
        Self::run_jaeger(self.config.clone());
        // Start prometheus http server
        Self::run_prometheus_server(self.config.clone());
        #[cfg(all(
            not(target_env = "msvc"),
            not(target_os = "macos"),
            feature = "jemalloc"
        ))]
        tokio::spawn(common_memory_tracker::track_current_process());

        log::info!("node starts");
        observe_listen_port_occupancy(&[self.config.network.listening_address.clone()]).await?;
        let config = self.config.clone();
        // Init Block db
        let path_block = config.data_path_for_block();
        log::info!("Data path for block: {:?}", path_block);

        let rocks_adapter = Arc::new(RocksAdapter::new(
            path_block.clone(),
            config.rocksdb.clone(),
        )?);

        #[cfg(all(
            not(target_env = "msvc"),
            not(target_os = "macos"),
            feature = "jemalloc"
        ))]
        tokio::spawn(common_memory_tracker::track_db_process(
            "blockdb",
            rocks_adapter.inner_db(),
        ));

        let storage = Arc::new(ImplStorage::new(
            rocks_adapter,
            self.config.rocksdb.cache_size,
        ));

        // Init network
        let network_config = NetworkConfig::new()
            .max_connections(config.network.max_connected_peers)?
            .peer_store_dir(config.data_path.clone().join("peer_store"))
            .ping_interval(config.network.ping_interval)
            .max_frame_length(config.network.max_frame_length)
            .send_buffer_size(config.network.send_buffer_size)
            .recv_buffer_size(config.network.recv_buffer_size);

        let network_privkey = config.privkey.as_string_trim0x();

        // let allowlist = config.network.allowlist.clone().unwrap_or_default();
        let network_config = network_config
            .bootstraps(self.config.network.bootstraps.clone().unwrap_or_default().iter().map(|addr| addr.multi_address.clone()).collect())
            // .allowlist(allowlist)?
            .listen_addr(self.config.network.listening_address.clone())
            .secio_keypair(&network_privkey)?;

        let mut network_service = NetworkService::new(network_config);
        network_service.set_chain_id(self.genesis.block.header.chain_id.to_string());

        // Init trie db
        let path_state = config.data_path_for_state();
        let trie_db = Arc::new(RocksTrieDB::new(
            path_state,
            config.rocksdb.clone(),
            config.executor.triedb_cache_size,
        )?);

        #[cfg(all(
            not(target_env = "msvc"),
            not(target_os = "macos"),
            feature = "jemalloc"
        ))]
        tokio::spawn(common_memory_tracker::track_db_process(
            "triedb",
            trie_db.inner_db(),
        ));

        // Init full transactions wal
        let txs_wal_path = config.data_path_for_txs_wal().to_str().unwrap().to_string();
        let txs_wal = Arc::new(SignedTxsWAL::new(txs_wal_path));

        // Init consensus wal
        let consensus_wal_path = config
            .data_path_for_consensus_wal()
            .to_str()
            .unwrap()
            .to_string();
        let consensus_wal = Arc::new(ConsensusWal::new(consensus_wal_path));

        // Recover signed transactions of current number
        let current_block = storage.get_latest_block(Context::new()).await?;
        let current_stxs = txs_wal.load_by_number(current_block.header.number + 1);
        log::info!(
            "Recover {} tx of number {} from wal",
            current_stxs.len(),
            current_block.header.number + 1
        );

        let metadata_adapter = MetadataAdapterImpl::new(Arc::clone(&storage), Arc::clone(&trie_db));
        let metadata_controller = Arc::new(MetadataController::new(
            Arc::new(metadata_adapter),
            self.config.epoch_len,
        ));

        let metadata = metadata_controller.get_metadata(Context::new(), &current_block.header)?;

        let ckb_client = Arc::new(RpcClient::new(
            &self.config.cross_client.ckb_uri,
            &self.config.cross_client.mercury_uri,
            &self.config.cross_client.indexer_uri.clone(),
        ));

        let interoperation = Arc::new(
            InteroperationImpl::new(
                self.config.interoperability_extension.clone().into(),
                Arc::clone(&ckb_client),
            )
            .await?,
        );

        // Init mempool
        let mempool_adapter = DefaultMemPoolAdapter::<Secp256k1, _, _, _, _, _>::new(
            network_service.handle(),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
            Arc::clone(&metadata_controller),
            Arc::clone(&interoperation),
            self.genesis.block.header.chain_id,
            self.genesis.block.header.gas_limit.as_u64(),
            config.mempool.pool_size as usize,
            config.mempool.broadcast_txs_size,
            config.mempool.broadcast_txs_interval,
        );
        let mempool = Arc::new(
            MemPoolImpl::new(
                config.mempool.pool_size as usize,
                config.mempool.timeout_gap,
                mempool_adapter,
                current_stxs.clone(),
            )
            .await,
        );

        let monitor_mempool = Arc::clone(&mempool);
        tokio::spawn(async move {
            let interval = Duration::from_millis(1000);
            loop {
                sleep(interval).await;
                MEMPOOL_LEN_GAUGE.set(monitor_mempool.len() as i64);
                MEMPOOL_CO_QUEUE_LEN.set(monitor_mempool.len() as i64);
            }
        });

        // self private key
        let hex_privkey = hex_decode(&config.privkey.as_string_trim0x())?;
        let my_privkey =
            Secp256k1PrivateKey::try_from(hex_privkey.as_ref()).map_err(MainError::Crypto)?;
        let my_pubkey = my_privkey.pub_key();
        let my_address = Address::from_pubkey_bytes(my_pubkey.to_uncompressed_bytes())?;

        // register broadcast new transaction
        network_service.register_endpoint_handler(
            END_GOSSIP_NEW_TXS,
            NewTxsHandler::new(Arc::clone(&mempool)),
        )?;

        // register pull txs from other node
        network_service.register_endpoint_handler(
            RPC_PULL_TXS,
            PullTxsHandler::new(Arc::new(network_service.handle()), Arc::clone(&mempool)),
        )?;
        network_service.register_rpc_response(RPC_RESP_PULL_TXS)?;

        network_service.register_rpc_response(RPC_RESP_PULL_TXS_SYNC)?;

        // Init Consensus
        let validators: Vec<Validator> = metadata
            .verifier_list
            .iter()
            .map(|v| Validator {
                pub_key:        v.pub_key.as_bytes(),
                propose_weight: v.propose_weight,
                vote_weight:    v.vote_weight,
            })
            .collect();

        let node_info = NodeInfo {
            chain_id:     self.genesis.block.header.chain_id,
            self_address: my_address.clone(),
            self_pub_key: my_pubkey.to_bytes(),
        };
        let current_header = &current_block.header;
        let current_number = current_block.header.number;
        let latest_proof = storage.get_latest_proof(Context::new()).await?;

        let current_consensus_status = CurrentStatus {
            prev_hash:                  current_block.hash(),
            last_number:                current_header.number,
            max_tx_size:                metadata.max_tx_size.into(),
            tx_num_limit:               metadata.tx_num_limit,
            last_checkpoint_block_hash: metadata.last_checkpoint_block_hash,
            proof:                      latest_proof,
            last_state_root:            if current_number == 0 {
                self.state_root
            } else {
                current_header.state_root
            },
        };

        CURRENT_STATE_ROOT.swap(Arc::new(current_consensus_status.last_state_root));
        CHAIN_ID.swap(Arc::new(current_header.chain_id));

        // set args in mempool
        mempool.set_args(
            Context::new(),
            current_header.state_root,
            metadata.gas_limit,
            metadata.max_tx_size,
        );

        // start ckb tx assembler
        let indexer_adapter = IndexerAdapter::new(Arc::clone(&ckb_client));
        let ckb_tx_assembler = Arc::new(TxAssemblerImpl::new(Arc::new(indexer_adapter)));
        let metadata_type_id = H256::from_slice(
            &hex_decode("490d951fe6d4d34d0c4f238b50b8b1d524ddf737275b1a1f1e3216f0af5c522e")
                .unwrap(),
        );
        let _ = ckb_tx_assembler
            .update_metadata(
                metadata_type_id,
                Default::default(),
                current_header.chain_id as u16,
                self.config.cross_client.enable,
            )
            .await?;

        // start cross chain client
        let path_crosschain = self.config.data_path_for_crosschain();
        let crosschain_db = CrossChainDBImpl::new(path_crosschain, config.rocksdb.clone())?;

        let crosschain_adapter = DefaultCrossChainAdapter::new(
            Arc::clone(&mempool),
            Arc::clone(&metadata_controller),
            Arc::clone(&storage),
            Arc::clone(&ckb_tx_assembler),
            Arc::clone(&trie_db),
            Arc::new(crosschain_db),
            Arc::clone(&ckb_client),
        )
        .await;

        let (crosschain_process, cross_handle, crosschain_net_handle) = CrossChainImpl::new(
            &hex_privkey,
            config.cross_client.clone(),
            Arc::clone(&ckb_client),
            Arc::new(crosschain_adapter),
        )
        .await;

        // start cross chain client
        if self.config.cross_client.enable {
            tokio::spawn(crosschain_process.run());
        }

        let consensus_interval = metadata.interval;
        let status_agent = StatusAgent::new(current_consensus_status);

        let mut bls_pub_keys = HashMap::new();
        for validator_extend in metadata.verifier_list.iter() {
            let address = validator_extend.pub_key.as_bytes();
            let hex_pubkey = hex_decode(&validator_extend.bls_pub_key.as_string_trim0x())?;
            let pub_key = BlsPublicKey::try_from(hex_pubkey.as_ref()).map_err(MainError::Crypto)?;
            bls_pub_keys.insert(address, pub_key);
        }

        let bls_priv_key =
            BlsPrivateKey::try_from(hex_privkey.as_ref()).map_err(MainError::Crypto)?;

        let crypto = Arc::new(OverlordCrypto::new(
            bls_priv_key,
            bls_pub_keys,
            String::new(),
        ));

        let consensus_adapter = OverlordConsensusAdapter::<_, _, _, _, _, _>::new(
            Arc::new(network_service.handle()),
            Arc::clone(&mempool),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
            Arc::new(cross_handle),
            Arc::clone(&metadata_controller),
            Arc::clone(&crypto),
        )?;

        let consensus_adapter = Arc::new(consensus_adapter);

        let lock = Arc::new(AsyncMutex::new(()));

        let overlord_consensus = Arc::new(OverlordConsensus::new(
            status_agent.clone(),
            self.config.metadata_contract_address.into(),
            node_info,
            Arc::clone(&crypto),
            Arc::clone(&txs_wal),
            Arc::clone(&consensus_adapter),
            Arc::clone(&lock),
            Arc::clone(&consensus_wal),
            self.config.cross_client.checkpoint_interval,
        ));

        consensus_adapter.set_overlord_handler(overlord_consensus.get_overlord_handler());

        let synchronization = Arc::new(OverlordSynchronization::<_>::new(
            config.consensus.sync_txs_chunk_size,
            consensus_adapter,
            status_agent.clone(),
            lock,
        ));

        let peer_ids = metadata
            .verifier_list
            .iter()
            .map(|v| PeerId::from_pubkey_bytes(v.pub_key.as_bytes()).map(PeerIdExt::into_bytes_ext))
            .collect::<Result<Vec<_>, _>>()?;

        network_service
            .handle()
            .tag_consensus(Context::new(), peer_ids)?;

        network_service.register_endpoint_handler(
            END_GOSSIP_CKB_TX_SIGNATURE,
            CrossChainMessageHandler::new(crosschain_net_handle.clone()),
        )?;

        network_service.register_endpoint_handler(
            END_GOSSIP_BUILD_CKB_TX,
            CrossChainMessageHandler::new(crosschain_net_handle),
        )?;

        // register consensus
        network_service.register_endpoint_handler(
            END_GOSSIP_SIGNED_PROPOSAL,
            ProposalMessageHandler::new(Arc::clone(&overlord_consensus)),
        )?;
        network_service.register_endpoint_handler(
            END_GOSSIP_AGGREGATED_VOTE,
            QCMessageHandler::new(Arc::clone(&overlord_consensus)),
        )?;
        network_service.register_endpoint_handler(
            END_GOSSIP_SIGNED_VOTE,
            VoteMessageHandler::new(Arc::clone(&overlord_consensus)),
        )?;
        network_service.register_endpoint_handler(
            END_GOSSIP_SIGNED_CHOKE,
            ChokeMessageHandler::new(Arc::clone(&overlord_consensus)),
        )?;
        network_service.register_endpoint_handler(
            BROADCAST_HEIGHT,
            RemoteHeightMessageHandler::new(Arc::clone(&synchronization)),
        )?;
        network_service.register_endpoint_handler(
            RPC_SYNC_PULL_BLOCK,
            PullBlockRpcHandler::new(Arc::new(network_service.handle()), Arc::clone(&storage)),
        )?;

        network_service.register_endpoint_handler(
            RPC_SYNC_PULL_PROOF,
            PullProofRpcHandler::new(Arc::new(network_service.handle()), Arc::clone(&storage)),
        )?;

        network_service.register_endpoint_handler(
            RPC_SYNC_PULL_TXS,
            PullTxsRpcHandler::new(Arc::new(network_service.handle()), Arc::clone(&storage)),
        )?;
        network_service.register_rpc_response(RPC_RESP_SYNC_PULL_BLOCK)?;
        network_service.register_rpc_response(RPC_RESP_SYNC_PULL_PROOF)?;
        network_service.register_rpc_response(RPC_RESP_SYNC_PULL_TXS)?;

        let network_handle = network_service.handle();

        // Run IBC
        // use core_ibc::run_ibc_grpc;
        // use core_ibc::DefaultIbcAdapter;
        // let _ibc_adapter =
        //     DefaultIbcAdapter::new(Arc::clone(&storage),
        // Arc::clone(&metadata_controller)).await; let grpc_addr =
        // "[::1]:50051".to_string(); tokio::spawn(async {
        //     run_ibc_grpc(_ibc_adapter, grpc_addr).await;
        // });

        // Run network
        tokio::spawn(network_service.run());

        // Run API
        let api_adapter = Arc::new(DefaultAPIAdapter::new(
            Arc::clone(&mempool),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
            Arc::new(network_handle),
        ));
        let _handles = run_jsonrpc_server(self.config.clone(), api_adapter).await?;

        // Run sync
        tokio::spawn(async move {
            if let Err(e) = synchronization.polling_broadcast().await {
                log::error!("synchronization: {:?}", e);
            }
        });

        // Run consensus
        let authority_list = validators
            .iter()
            .map(|v| Node {
                address:        v.pub_key.clone(),
                propose_weight: v.propose_weight,
                vote_weight:    v.vote_weight,
            })
            .collect::<Vec<_>>();

        let timer_config = DurationConfig {
            propose_ratio:   metadata.propose_ratio,
            prevote_ratio:   metadata.prevote_ratio,
            precommit_ratio: metadata.precommit_ratio,
            brake_ratio:     metadata.brake_ratio,
        };

        tokio::spawn(async move {
            if let Err(e) = overlord_consensus
                .run(
                    current_number,
                    consensus_interval,
                    authority_list,
                    Some(timer_config),
                )
                .await
            {
                log::error!("axon-consensus: {:?} error", e);
            }
        });

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

        Ok(())
    }

    fn run_jaeger(config: Config) {
        if let Some(jaeger_config) = config.jaeger {
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
            log::info!("jaeger start");
        };
    }

    fn run_prometheus_server(config: Config) {
        let prometheus_listening_address = match config.prometheus {
            Some(prometheus_config) => prometheus_config.listening_address.unwrap(),
            None => std::net::SocketAddr::from(([0, 0, 0, 0], 8100)),
        };
        tokio::spawn(run_prometheus_server(prometheus_listening_address));

        log::info!("prometheus start");
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

    #[display(fmt = "{:?}", _0)]
    WalletError(ethers_signers::WalletError),

    #[display(fmt = "other error {:?}", _0)]
    Other(String),
}

impl std::error::Error for MainError {}

impl From<MainError> for ProtocolError {
    fn from(error: MainError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Main, Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use protocol::codec::hex_decode;
    use protocol::types::RichBlock;
    use std::fs;

    #[test]
    fn decode_genesis() {
        let raw = fs::read("../../devtools/chain/genesis_single_node.json").unwrap();
        let genesis: RichBlock = serde_json::from_slice(&raw).unwrap();
        println!("{:?}", genesis);
    }

    #[test]
    fn decode_type_id() {
        let type_id =
            hex_decode("c0810210210c06ba233273e94d7fc89b00a705a07fdc0ae4c78e4036582ff336").unwrap();
        println!("{:?}", type_id);
    }
}
