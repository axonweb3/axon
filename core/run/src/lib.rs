#![feature(async_closure, once_cell)]
#![allow(clippy::mutable_key_type)]

use std::collections::HashMap;
use std::convert::TryFrom;
use std::panic;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use backtrace::Backtrace;
use parking_lot::Mutex;

use common_apm::muta_apm;
use common_config_parser::types::Config;
use common_crypto::{
    BlsCommonReference, BlsPrivateKey, BlsPublicKey, PublicKey, Secp256k1, Secp256k1PrivateKey,
    ToPublicKey, UncompressedPublicKey,
};
use core_consensus::message::{
    ChokeMessageHandler, ProposalMessageHandler, PullBlockRpcHandler, PullProofRpcHandler,
    PullTxsRpcHandler, QCMessageHandler, RemoteHeightMessageHandler, VoteMessageHandler,
    BROADCAST_HEIGHT, END_GOSSIP_AGGREGATED_VOTE, END_GOSSIP_SIGNED_CHOKE,
    END_GOSSIP_SIGNED_PROPOSAL, END_GOSSIP_SIGNED_VOTE, RPC_RESP_SYNC_PULL_BLOCK,
    RPC_RESP_SYNC_PULL_PROOF, RPC_RESP_SYNC_PULL_TXS, RPC_SYNC_PULL_BLOCK, RPC_SYNC_PULL_PROOF,
    RPC_SYNC_PULL_TXS,
};
use core_consensus::status::{CurrentStatus, MetadataController, StatusAgent};
use core_consensus::{engine::generate_receipts_and_logs, util::OverlordCrypto};
use core_consensus::{
    ConsensusWal, DurationConfig, Node, OverlordConsensus, OverlordConsensusAdapter,
    OverlordSynchronization, SignedTxsWAL, METADATA_CONTROLER,
};
use core_executor::adapter::{trie_db::RocksTrieDB, ExecutorAdapter};
use core_executor::EvmExecutor;
use core_mempool::{
    DefaultMemPoolAdapter, HashMemPool, NewTxsHandler, PullTxsHandler, END_GOSSIP_NEW_TXS,
    RPC_PULL_TXS, RPC_RESP_PULL_TXS, RPC_RESP_PULL_TXS_SYNC,
};
use core_network::{NetworkConfig, NetworkService};
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
#[cfg(unix)]
use protocol::tokio::signal::unix::{self as os_impl};
use protocol::tokio::{sync::Mutex as AsyncMutex, time::sleep};
use protocol::traits::{CommonStorage, Context, Executor, MemPool, NodeInfo, Storage};
use protocol::types::{Address, Bloom, BloomInput, Genesis, Metadata, Validator};
use protocol::{tokio, Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult};

#[derive(Debug)]
pub struct Axon {
    config:   Config,
    genesis:  Genesis,
    metadata: Metadata,
}

impl Axon {
    pub fn new(config: Config, genesis: Genesis, metadata: Metadata) -> Axon {
        Axon {
            config,
            genesis,
            metadata,
        }
    }

    pub fn run(self) -> ProtocolResult<()> {
        if let Some(apm_config) = &self.config.apm {
            muta_apm::global_tracer_register(
                &apm_config.service_name,
                apm_config.tracing_address,
                apm_config.tracing_batch_size,
            );

            log::info!("muta_apm start");
        }

        let rt = tokio::runtime::Runtime::new().expect("new tokio runtime");
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, async move {
            self.create_genesis().await?;
            self.start().await
        })?;

        Ok(())
    }

    pub async fn create_genesis(&self) -> ProtocolResult<()> {
        log::info!("Genesis data: {:?}", self.genesis);

        // Init Block db
        let path_block = self.config.data_path_for_block();
        let rocks_adapter = Arc::new(RocksAdapter::new(
            path_block,
            self.config.rocksdb.max_open_files,
        )?);
        let storage = Arc::new(ImplStorage::new(rocks_adapter));

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
            self.config.rocksdb.max_open_files,
            self.config.executor.triedb_cache_size,
        )?);

        // Init executor
        let executor = EvmExecutor::default();
        let mut backend = ExecutorAdapter::new(
            trie_db,
            Arc::new(Mutex::new(self.genesis.block.header.clone().into())),
        )?;

        log::info!("Execute the genesis");

        let resp = executor.exec(&mut backend, self.genesis.rich_txs.clone());

        let (receipts, _logs) = generate_receipts_and_logs(
            self.genesis.block.header.state_root,
            &self.genesis.rich_txs,
            &resp,
        );

        storage
            .update_latest_proof(Context::new(), self.genesis.block.header.proof.clone())
            .await?;
        storage
            .insert_transactions(
                Context::new(),
                self.genesis.block.header.number,
                self.genesis.rich_txs.clone(),
            )
            .await?;
        storage
            .insert_receipts(Context::new(), self.genesis.block.header.number, receipts)
            .await?;
        storage
            .insert_block(Context::new(), self.genesis.block.clone())
            .await?;

        log::info!("The genesis block is created {:?}", self.genesis);
        Ok(())
    }

    pub async fn start(self) -> ProtocolResult<()> {
        log::info!("node starts");
        let config = self.config.clone();
        // Init Block db
        let path_block = config.data_path_for_block();
        log::info!("Data path for block: {:?}", path_block);

        let rocks_adapter = Arc::new(RocksAdapter::new(
            path_block.clone(),
            config.rocksdb.max_open_files,
        )?);
        let storage = Arc::new(ImplStorage::new(Arc::clone(&rocks_adapter)));

        // Init network
        let network_config = NetworkConfig::new()
            .max_connections(config.network.max_connected_peers)?
            // .same_ip_conn_limit(config.network.same_ip_conn_limit)
            // .inbound_conn_limit(config.network.inbound_conn_limit)?
            // .allowlist_only(config.network.allowlist_only)
            // .peer_trust_metric(
            //     config.network.trust_interval_duration,
            //     config.network.trust_max_history_duration,
            // )?
            // .peer_soft_ban(config.network.soft_ban_duration)
            // .peer_fatal_ban(config.network.fatal_ban_duration)
            // .rpc_timeout(config.network.rpc_timeout)
            .ping_interval(config.network.ping_interval)
            // .selfcheck_interval(config.network.selfcheck_interval)
            // .max_wait_streams(config.network.max_wait_streams)
            .max_frame_length(config.network.max_frame_length)
            .send_buffer_size(config.network.send_buffer_size)
            // .write_timeout(config.network.write_timeout)
            .recv_buffer_size(config.network.recv_buffer_size);

        let network_privkey = config.privkey.as_string_trim0x();

        // let allowlist = config.network.allowlist.clone().unwrap_or_default();
        let network_config = network_config
            .bootstraps(self.config.network.bootstraps.clone().unwrap_or_default().iter().map(|addr| addr.multi_address.clone()).collect())
            // .allowlist(allowlist)?
            .listen_addr(self.config.network.listening_address.clone())
            .secio_keypair(network_privkey)?;

        let mut network_service = NetworkService::new(network_config);

        // Init trie db
        let path_state = config.data_path_for_state();
        let trie_db = Arc::new(RocksTrieDB::new(
            path_state,
            config.rocksdb.max_open_files,
            config.executor.triedb_cache_size,
        )?);

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

        // Init mempool
        let mempool_adapter = DefaultMemPoolAdapter::<Secp256k1, _, _>::new(
            network_service.handle(),
            Arc::clone(&storage),
            self.genesis.block.header.chain_id,
            config.mempool.timeout_gap,
            self.genesis.block.header.gas_limit.as_u64(),
            config.mempool.pool_size as usize,
            config.mempool.broadcast_txs_size,
            config.mempool.broadcast_txs_interval,
        );
        let mempool = Arc::new(
            HashMemPool::new(
                config.mempool.pool_size as usize,
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
                common_apm::metrics::mempool::MEMPOOL_LEN_GAUGE
                    .set(monitor_mempool.get_tx_cache().len().await as i64);
            }
        });

        // self private key
        let hex_privkey =
            hex::decode(config.privkey.as_string_trim0x()).map_err(MainError::FromHex)?;
        let my_privkey =
            Secp256k1PrivateKey::try_from(hex_privkey.as_ref()).map_err(MainError::Crypto)?;
        let my_pubkey = my_privkey.pub_key();
        let my_address = Address::from_pubkey_bytes(my_pubkey.to_uncompressed_bytes())?;

        METADATA_CONTROLER
            .set(MetadataController::init(
                Arc::new(Mutex::new(self.metadata.clone())),
                Arc::new(Mutex::new(self.metadata.clone())),
                Arc::new(Mutex::new(self.metadata.clone())),
            ))
            .unwrap();

        let metadata = METADATA_CONTROLER.get().unwrap().current();

        // set args in mempool
        mempool.set_args(
            Context::new(),
            metadata.timeout_gap,
            metadata.gas_limit,
            metadata.max_tx_size,
        );

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
                pub_key:        v.pub_key.decode(),
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

        // Init executor
        let executor = EvmExecutor::default();
        let mut backend = if current_header.state_root == Default::default() {
            ExecutorAdapter::new(
                Arc::clone(&trie_db),
                Arc::new(Mutex::new(current_header.clone().into())),
            )
        } else {
            ExecutorAdapter::from_root(
                current_header.state_root,
                Arc::clone(&trie_db),
                Arc::new(Mutex::new(current_header.clone().into())),
            )
        }?;
        let resp = executor.exec(&mut backend, current_stxs.clone());

        let (_receipts, logs) = generate_receipts_and_logs(
            self.genesis.block.header.state_root,
            &self.genesis.rich_txs,
            &resp,
        );

        let current_consensus_status = CurrentStatus {
            prev_hash:        current_header.prev_hash,
            last_number:      current_header.number,
            state_root:       resp.state_root,
            receipts_root:    resp.receipt_root,
            log_bloom:        Bloom::from(BloomInput::Raw(rlp::encode_list(&logs).as_ref())),
            gas_used:         resp.gas_used.into(),
            gas_limit:        metadata.gas_limit.into(),
            base_fee_per_gas: None,
            proof:            current_header.proof.clone(),
        };

        let consensus_interval = metadata.interval;
        let status_agent = StatusAgent::new(current_consensus_status);

        let mut bls_pub_keys = HashMap::new();
        for validator_extend in metadata.verifier_list.iter() {
            let address = validator_extend.pub_key.decode();
            let hex_pubkey = hex::decode(validator_extend.bls_pub_key.as_string_trim0x())
                .map_err(MainError::FromHex)?;
            let pub_key = BlsPublicKey::try_from(hex_pubkey.as_ref()).map_err(MainError::Crypto)?;
            bls_pub_keys.insert(address, pub_key);
        }

        let mut priv_key = Vec::new();
        priv_key.extend_from_slice(&[0u8; 16]);
        let mut tmp = hex::decode(config.privkey.as_string_trim0x()).unwrap();
        priv_key.append(&mut tmp);
        let bls_priv_key = BlsPrivateKey::try_from(priv_key.as_ref()).map_err(MainError::Crypto)?;

        let hex_common_ref =
            hex::decode(metadata.common_ref.as_string_trim0x()).map_err(MainError::FromHex)?;
        let common_ref: BlsCommonReference = std::str::from_utf8(hex_common_ref.as_ref())
            .map_err(MainError::Utf8)?
            .into();

        let crypto = Arc::new(OverlordCrypto::new(bls_priv_key, bls_pub_keys, common_ref));

        let consensus_adapter = OverlordConsensusAdapter::<_, _, _, _>::new(
            Arc::new(network_service.handle()),
            Arc::clone(&mempool),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
            Arc::clone(&crypto),
        )?;

        let consensus_adapter = Arc::new(consensus_adapter);

        let lock = Arc::new(AsyncMutex::new(()));

        let overlord_consensus = Arc::new(OverlordConsensus::new(
            status_agent.clone(),
            node_info,
            Arc::clone(&crypto),
            Arc::clone(&txs_wal),
            Arc::clone(&consensus_adapter),
            Arc::clone(&lock),
            Arc::clone(&consensus_wal),
        ));

        consensus_adapter.set_overlord_handler(overlord_consensus.get_overlord_handler());

        let synchronization = Arc::new(OverlordSynchronization::<_>::new(
            config.consensus.sync_txs_chunk_size,
            consensus_adapter,
            status_agent.clone(),
            lock,
        ));

        // let peer_ids = metadata
        //     .verifier_list
        //     .iter()
        //     .map(|v|
        // PeerId::from_pubkey_bytes(v.pub_key.decode()).map(PeerIdExt::into_bytes_ext))
        //     .collect::<Result<Vec<_>, _>>()?;

        // network_service
        //     .handle()
        //     .tag_consensus(Context::new(), peer_ids)?;

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

        // Run network
        tokio::spawn(network_service.run());

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
                log::error!("muta-consensus: {:?} error", e);
            }
        });

        let ctrl_c_handler = tokio::task::spawn_local(async {
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

        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            let panic_sender = panic_sender.clone();
            Self::panic_log(info);
            panic_sender.try_send(()).expect("panic_receiver is droped");
        }));

        tokio::select! {
            _ = ctrl_c_handler =>{ log::info!("ctrl + c is pressed, quit.") },
            _ = panic_receiver.recv() => { log::info!("child thraed panic, quit.")},
        };

        Ok(())
    }

    fn panic_log(info: &panic::PanicInfo) {
        let backtrace = Backtrace::new();
        let thread = thread::current();
        let name = thread.name().unwrap_or("unnamed");
        let location = info.location().unwrap(); // The current implementation always returns Some
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &*s,
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
}

#[derive(Debug, Display, From)]
pub enum MainError {
    #[display(fmt = "The muta configuration read failed {:?}", _0)]
    ConfigParse(common_config_parser::ParseError),

    #[display(fmt = "{:?}", _0)]
    Io(std::io::Error),

    #[display(fmt = "Toml fails to parse genesis {:?}", _0)]
    GenesisTomlDe(toml::de::Error),

    #[display(fmt = "hex error {:?}", _0)]
    FromHex(hex::FromHexError),

    #[display(fmt = "crypto error {:?}", _0)]
    Crypto(common_crypto::Error),

    #[display(fmt = "{:?}", _0)]
    Utf8(std::str::Utf8Error),

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
