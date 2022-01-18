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
    BlsPrivateKey, BlsPublicKey, PublicKey, Secp256k1, Secp256k1PrivateKey,
    Secp256k1RecoverablePrivateKey, ToPublicKey, UncompressedPublicKey,
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
use core_consensus::status::{CurrentStatus, MetadataController, StatusAgent};
use core_consensus::{
    util::OverlordCrypto, ConsensusWal, DurationConfig, Node, OverlordConsensus,
    OverlordConsensusAdapter, OverlordSynchronization, SignedTxsWAL, METADATA_CONTROLER,
};
use core_cross_client::DefaultCrossAdapter;
use core_executor::{EVMExecutorAdapter, EvmExecutor, MPTTrie, RocksTrieDB};
use core_mempool::{
    DefaultMemPoolAdapter, MemPoolImpl, NewTxsHandler, PullTxsHandler, END_GOSSIP_NEW_TXS,
    RPC_PULL_TXS, RPC_RESP_PULL_TXS, RPC_RESP_PULL_TXS_SYNC,
};
use core_network::{
    observe_listen_port_occupancy, NetworkConfig, NetworkService, PeerId, PeerIdExt,
};
use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
use protocol::codec::{hex_decode, ProtocolCodec};
use protocol::lazy::{ASSET_CONTRACT_ADDRESS, CHAIN_ID, CURRENT_STATE_ROOT};
#[cfg(unix)]
use protocol::tokio::signal::unix as os_impl;
use protocol::tokio::{runtime::Builder as RuntimeBuilder, sync::Mutex as AsyncMutex, time::sleep};
use protocol::traits::{CommonStorage, Context, Executor, MemPool, Network, NodeInfo, Storage};
use protocol::types::{
    Account, Address, Hasher, MerkleRoot, Metadata, Proposal, RichBlock, Validator, NIL_DATA,
    RLP_NULL, U256,
};
use protocol::{tokio, Display, From, ProtocolError, ProtocolErrorKind, ProtocolResult};

#[derive(Debug)]
pub struct Axon {
    config:     Config,
    genesis:    RichBlock,
    metadata:   Metadata,
    state_root: MerkleRoot,
}

impl Axon {
    pub fn new(config: Config, genesis: RichBlock, metadata: Metadata) -> Axon {
        Axon {
            config,
            genesis,
            metadata,
            state_root: MerkleRoot::default(),
        }
    }

    pub fn run(mut self) -> ProtocolResult<()> {
        if let Some(apm_config) = &self.config.apm {
            muta_apm::global_tracer_register(
                &apm_config.service_name,
                apm_config.tracing_address,
                apm_config.tracing_batch_size,
            );

            log::info!("muta_apm start");
        }

        let rt = RuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .expect("new tokio runtime");

        rt.block_on(async move {
            self.create_genesis().await?;
            self.start().await
        })
    }

    pub async fn create_genesis(&mut self) -> ProtocolResult<()> {
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
        let mut mpt = MPTTrie::new(Arc::clone(&trie_db));

        let distribute_address = Address::from_hex("0x8ab0cf264df99d83525e9e11c7e4db01558ae1b1")?;
        let distribute_account = Account {
            nonce:        0u64.into(),
            balance:      32000001100000000000u128.into(),
            storage_root: RLP_NULL,
            code_hash:    NIL_DATA,
        };

        mpt.insert(
            distribute_address.as_slice(),
            distribute_account.encode()?.as_ref(),
        )?;

        let proposal = Proposal::from(self.genesis.block.clone());
        let executor = EvmExecutor::default();
        let mut backend = EVMExecutorAdapter::from_root(
            mpt.commit()?,
            trie_db,
            Arc::clone(&storage),
            proposal.into(),
        )?;
        let resp = executor.exec(&mut backend, self.genesis.txs.clone());

        self.state_root = resp.state_root;

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
        log::info!("node starts");
        observe_listen_port_occupancy(&[self.config.network.listening_address.clone()]).await?;
        let config = self.config.clone();
        // Init Block db
        let path_block = config.data_path_for_block();
        log::info!("Data path for block: {:?}", path_block);

        let rocks_adapter = Arc::new(RocksAdapter::new(
            path_block.clone(),
            config.rocksdb.max_open_files,
        )?);
        let storage = Arc::new(ImplStorage::new(rocks_adapter));

        // Init network
        let network_config = NetworkConfig::new()
            .max_connections(config.network.max_connected_peers)?
            .peer_store_dir(config.data_path.clone().join("peer_store"))
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
            .secio_keypair(&network_privkey)?;

        let mut network_service = NetworkService::new(network_config);
        network_service.set_chain_id(self.genesis.block.header.chain_id.to_string());

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
        let mempool_adapter = DefaultMemPoolAdapter::<Secp256k1, _, _, _>::new(
            network_service.handle(),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
            self.genesis.block.header.chain_id,
            config.mempool.timeout_gap,
            self.genesis.block.header.gas_limit.as_u64(),
            config.mempool.pool_size as usize,
            config.mempool.broadcast_txs_size,
            config.mempool.broadcast_txs_interval,
        );
        let mempool = Arc::new(
            MemPoolImpl::new(
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
                common_apm::metrics::mempool::MEMPOOL_LEN_GAUGE.set(monitor_mempool.len() as i64);
            }
        });

        // self private key
        let hex_privkey = hex_decode(&config.privkey.as_string_trim0x())?;
        let my_privkey =
            Secp256k1PrivateKey::try_from(hex_privkey.as_ref()).map_err(MainError::Crypto)?;
        let my_pubkey = my_privkey.pub_key();
        let my_address = Address::from_pubkey_bytes(my_pubkey.to_uncompressed_bytes())?;

        METADATA_CONTROLER.swap(Arc::new(MetadataController::init(
            Arc::new(Mutex::new(self.metadata.clone())),
            Arc::new(Mutex::new(self.metadata.clone())),
            Arc::new(Mutex::new(self.metadata.clone())),
        )));

        let metadata = METADATA_CONTROLER.load().current();

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

        let current_consensus_status = if current_number == 0 {
            CurrentStatus {
                prev_hash:        Hasher::digest(current_header.encode()?),
                last_number:      current_header.number,
                last_state_root:  self.state_root,
                gas_limit:        metadata.gas_limit.into(),
                base_fee_per_gas: U256::one(),
                proof:            latest_proof,
            }
        } else {
            // Init executor
            let proposal = Proposal::from(current_header.clone());
            let executor = EvmExecutor::default();
            let mut backend = EVMExecutorAdapter::from_root(
                current_header.state_root,
                Arc::clone(&trie_db),
                Arc::clone(&storage),
                proposal.into(),
            )?;
            let _resp = executor.exec(&mut backend, current_stxs.clone());
            let block_hash = Hasher::digest(current_header.encode()?);

            CurrentStatus {
                prev_hash:        block_hash,
                last_number:      current_header.number,
                last_state_root:  current_header.state_root,
                gas_limit:        metadata.gas_limit.into(),
                base_fee_per_gas: current_header.base_fee_per_gas,
                proof:            current_header.proof.clone(),
            }
        };

        CURRENT_STATE_ROOT.swap(Arc::new(current_consensus_status.last_state_root));
        CHAIN_ID.swap(Arc::new(current_header.chain_id));
        ASSET_CONTRACT_ADDRESS.swap(Arc::new(self.config.asset_contract_address.into()));

        // set args in mempool
        mempool.set_args(
            Context::new(),
            current_header.state_root,
            metadata.timeout_gap,
            metadata.gas_limit,
            metadata.max_tx_size,
        );

        // start cross chain client
        let cross_client = DefaultCrossAdapter::new(
            self.config.clone(),
            Secp256k1RecoverablePrivateKey::try_from(hex_privkey.as_ref()).unwrap(),
            Arc::clone(&mempool),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
        );
        let cross_handle = cross_client.handle();

        // start cross chain client
        if self.config.cross_client.enable {
            tokio::spawn(cross_client.run());
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
        let hex_common_ref = hex_decode(&metadata.common_ref.as_string_trim0x())?;
        let common_ref = String::from_utf8(hex_common_ref).map_err(MainError::Utf8)?;

        let crypto = Arc::new(OverlordCrypto::new(bls_priv_key, bls_pub_keys, common_ref));

        let consensus_adapter = OverlordConsensusAdapter::<_, _, _, _, _>::new(
            Arc::new(network_service.handle()),
            Arc::clone(&mempool),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
            Arc::new(cross_handle),
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

        // Run API
        let api_adapter = Arc::new(DefaultAPIAdapter::new(
            Arc::clone(&mempool),
            Arc::clone(&storage),
            Arc::clone(&trie_db),
        ));
        let _handles = run_jsonrpc_server(self.config.rpc.clone(), api_adapter).await?;

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

        panic::set_hook(Box::new(move |info: &panic::PanicInfo| {
            let panic_sender = panic_sender.clone();
            Self::panic_log(info);
            panic_sender.try_send(()).expect("panic_receiver is droped");
        }));

        tokio::select! {
            _ = ctrl_c_handler => { log::info!("ctrl + c is pressed, quit.") },
            _ = panic_receiver.recv() => { log::info!("child thraed panic, quit.") },
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

#[cfg(test)]
mod tests {
    use protocol::types::RichBlock;
    use std::fs;

    #[test]
    fn decode_genesis() {
        let raw = fs::read("../../devtools/chain/genesis.json").unwrap();
        let genesis: RichBlock = serde_json::from_slice(&raw).unwrap();
        println!("{:?}", genesis);
    }
}
