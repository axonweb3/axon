use std::{collections::HashMap, sync::Arc, time::Duration};

use common_apm::metrics::mempool::{MEMPOOL_CO_QUEUE_LEN, MEMPOOL_LEN_GAUGE};
use common_config_parser::types::spec::{ChainSpec, InitialAccount};
use common_config_parser::types::{Config, ConfigMempool};
use common_crypto::{BlsPrivateKey, BlsPublicKey, Secp256k1, Secp256k1PrivateKey, ToPublicKey};

pub use core_consensus::stop_signal::StopOpt;
use core_consensus::stop_signal::StopSignal;
use protocol::tokio::{
    self, runtime::Builder as RuntimeBuilder, sync::Mutex as AsyncMutex, time::sleep,
};
use protocol::traits::{
    Context, Executor, Gossip, MemPool, Network, NodeInfo, PeerTrust, ReadOnlyStorage, Rpc, Storage,
};
use protocol::types::{
    Block, Bloom, BloomInput, ExecResp, HardforkInfoInner, Header, Metadata, Proposal, RichBlock,
    SignedTransaction, Validator, ValidatorExtend, H256,
};
use protocol::{lazy::CHAIN_ID, trie::DB as TrieDB, ProtocolResult};

use core_api::{jsonrpc::run_jsonrpc_server, DefaultAPIAdapter};
use core_consensus::status::{CurrentStatus, StatusAgent};
use core_consensus::{
    util::OverlordCrypto, ConsensusWal, DurationConfig, OverlordConsensus,
    OverlordConsensusAdapter, OverlordSynchronization, SignedTxsWAL,
};
use core_executor::system_contract::{self, metadata::MetadataHandle};
use core_executor::{AxonExecutor, AxonExecutorApplyAdapter, AxonExecutorReadOnlyAdapter, MPTTrie};
use core_interoperation::InteroperationImpl;
use core_mempool::{DefaultMemPoolAdapter, MemPoolImpl};
use core_network::{observe_listen_port_occupancy, NetworkConfig, NetworkService};

pub use core_network::{KeyProvider, SecioKeyPair};

pub(crate) mod components;
mod error;
mod key_provider;

#[cfg(test)]
mod tests;

use components::{
    chain_spec::ChainSpecExt as _,
    extensions::ExtensionConfig as _,
    network::NetworkServiceExt as _,
    storage::{DatabaseGroup, StorageExt as _, TrieExt as _},
};
pub use error::MainError;
use key_provider::KeyP;

pub fn init(config: Config, spec: ChainSpec) -> ProtocolResult<()> {
    let genesis = spec.generate_genesis_block();

    let path_rocksdb = config.data_path_for_rocksdb();
    if path_rocksdb.exists() {
        let msg = format!("Data directory {} already exists.", path_rocksdb.display());
        return Err(MainError::Other(msg).into());
    }

    let rt = RuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()
        .expect("new tokio runtime");

    rt.block_on(async move {
        log::info!("Load databases.");
        let db_group = DatabaseGroup::new(
            &config.rocksdb,
            path_rocksdb,
            true,
            config.executor.triedb_cache_size,
        )?;
        log::info!("Initialize genesis block.");
        execute_genesis(genesis, &spec, &db_group).await
    })?;

    Ok(())
}

pub fn run<K: KeyProvider>(
    version: String,
    config: Config,
    key_provider: Option<K>,
    stop_opt: Option<StopOpt>,
) -> ProtocolResult<()> {
    let path_rocksdb = config.data_path_for_rocksdb();
    if !path_rocksdb.exists() {
        let msg = format!(
            "Data directory {} doesn't exist, please initialize it before run.",
            path_rocksdb.display()
        );
        return Err(MainError::Other(msg).into());
    }
    let rt = RuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()
        .expect("new tokio runtime");

    rt.block_on(async move {
        log::info!("Load databases.");
        let db_group = DatabaseGroup::new(
            &config.rocksdb,
            path_rocksdb,
            false,
            config.executor.triedb_cache_size,
        )?;
        log::info!("Start all services.");
        start(version, config, key_provider, &db_group, stop_opt).await
    })?;
    rt.shutdown_timeout(std::time::Duration::from_secs(1));

    Ok(())
}

async fn start<K: KeyProvider>(
    version: String,
    config: Config,
    key_provider: Option<K>,
    db_group: &DatabaseGroup,
    stop_opt: Option<StopOpt>,
) -> ProtocolResult<()> {
    let storage = db_group.storage();
    let trie_db = db_group.trie_db();
    let inner_db = db_group.inner_db();

    components::profiling::start();
    components::profiling::track_db_process("blockdb", &inner_db);
    components::profiling::track_current_process();

    // Start jaeger
    config.jaeger.start_if_possible();

    // Start prometheus http server
    config.prometheus.start_if_possible();

    log::info!("node starts");

    observe_listen_port_occupancy(&[config.network.listening_address.clone()]).await?;

    // Init Block db and get the current block
    let current_block = storage.get_latest_block(Context::new()).await?;
    let current_state_root = current_block.header.state_root;

    log::info!("At block number {}", current_block.header.number + 1);

    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let stop_signal = match stop_opt {
        Some(opt) => {
            let height = match opt {
                StopOpt::MineNBlocks(n) => current_block.header.number + n,
                StopOpt::MineToHeight(height) => height,
            };
            StopSignal::with_stop_at(stop_tx, height)
        }
        None => StopSignal::new(stop_tx),
    };

    // Init network
    let mut network_service =
        init_network_service(&config, current_block.header.chain_id, key_provider)?;

    // Init full transactions wal
    let txs_wal_path = config
        .data_path_for_txs_wal()
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            let msg = format!(
                "failed to convert WAL path {} to string",
                config.data_path_for_txs_wal().display()
            );
            MainError::Other(msg)
        })?;
    let txs_wal = Arc::new(SignedTxsWAL::new(txs_wal_path));

    // Init system contract
    let mut backend = AxonExecutorApplyAdapter::from_root(
        current_block.header.state_root,
        Arc::clone(&trie_db),
        Arc::clone(&storage),
        Proposal::new_without_state_root(&current_block.header).into(),
    )?;

    // The first two metadata has been inserted in the init process, only need to
    // init the system contract DB here.
    system_contract::init_system_contract_db(inner_db, &mut backend);

    // Init mempool and recover signed transactions with the current block number
    let current_stxs = txs_wal.load_by_number(current_block.header.number + 1);
    log::info!("Recover {} txs from wal", current_stxs.len());

    let mempool =
        init_mempool(
            &config.mempool,
            &current_block.header,
            &storage,
            &trie_db,
            &network_service.handle(),
            &current_stxs,
        )
        .await;

    // Get the validator list from current metadata for consensus initialization
    let metadata_root = AxonExecutorReadOnlyAdapter::from_root(
        current_state_root,
        Arc::clone(&trie_db),
        Arc::clone(&storage),
        Proposal::new_without_state_root(&current_block.header).into(),
    )?
    .get_metadata_root();

    let metadata_handle = MetadataHandle::new(metadata_root);
    metadata_handle.init_hardfork(current_block.header.number)?;

    let metadata = metadata_handle.get_metadata_by_block_number(current_block.header.number)?;
    let validators: Vec<Validator> = metadata.verifier_list.iter().map(Into::into).collect();

    // Set args in mempool
    mempool.set_args(
        Context::new(),
        current_block.header.state_root,
        metadata.consensus_config.gas_limit,
        metadata.consensus_config.max_tx_size,
    );

    // Init overlord consensus and synchronization
    let lock = Arc::new(AsyncMutex::new(()));
    let crypto = init_crypto(config.bls_privkey.as_ref(), &metadata.verifier_list)?;
    let consensus_adapter = OverlordConsensusAdapter::<_, _, _, _>::new(
        Arc::new(network_service.handle()),
        Arc::clone(&mempool),
        Arc::clone(&storage),
        Arc::clone(&trie_db),
        Arc::clone(&crypto),
    )?;
    let consensus_adapter = Arc::new(consensus_adapter);
    let status_agent = get_status_agent(&storage, &current_block, &metadata).await?;

    let hardfork_info = storage.hardfork_proposal(Default::default()).await?;
    let overlord_consensus = {
        let consensus_wal_path = config.data_path_for_consensus_wal();
        let node_info =
            Secp256k1PrivateKey::try_from(config.net_privkey.as_ref())
                .map(|privkey| {
                    NodeInfo::new(
                        current_block.header.chain_id,
                        privkey.pub_key(),
                        hardfork_info,
                    )
                })
                .map_err(MainError::Crypto)?;
        let overlord_consensus = OverlordConsensus::new(
            status_agent.clone(),
            node_info,
            Arc::clone(&crypto),
            Arc::clone(&txs_wal),
            Arc::clone(&consensus_adapter),
            Arc::clone(&lock),
            Arc::new(ConsensusWal::new(consensus_wal_path)),
            stop_signal,
        )
        .await;
        Arc::new(overlord_consensus)
    };

    consensus_adapter.set_overlord_handler(overlord_consensus.get_overlord_handler());

    let synchronization = Arc::new(OverlordSynchronization::<_>::new(
        config.sync.sync_txs_chunk_size,
        consensus_adapter,
        status_agent.clone(),
        lock,
    ));

    network_service.tag_consensus(&metadata.verifier_list)?;

    // register endpoints to network service
    network_service.register_mempool_endpoint(&mempool)?;
    network_service.register_consensus_endpoint(&overlord_consensus)?;
    network_service.register_synchronization_endpoint(&synchronization)?;
    network_service.register_storage_endpoint(&storage)?;
    network_service.register_rpc()?;

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
    let _handles = run_jsonrpc_server(version, config, api_adapter).await?;

    // Run sync
    tokio::spawn(async move {
        if let Err(e) = synchronization.polling_broadcast().await {
            log::error!("synchronization: {:?}", e);
        }
    });

    // Run consensus
    run_overlord_consensus(metadata, validators, current_block, overlord_consensus);

    tokio::select! {
        () = components::system::set_ctrl_c_handle() => {}
        _ = stop_rx => {}
    }
    components::profiling::stop();

    Ok(())
}

fn init_network_service<K: KeyProvider>(
    config: &Config,
    chain_id: u64,
    key_provider: Option<K>,
) -> ProtocolResult<NetworkService<KeyP<K>>> {
    let network_config = NetworkConfig::from_config(config, chain_id)?;

    let key = key_provider
        .map(KeyP::Custom)
        .unwrap_or(KeyP::Default(network_config.secio_keypair.clone()));

    Ok(NetworkService::new(network_config, key))
}

async fn init_mempool<N, S, DB>(
    config: &ConfigMempool,
    current_header: &Header,
    storage: &Arc<S>,
    trie_db: &Arc<DB>,
    network_service: &N,
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
        current_header.chain_id,
        current_header.gas_limit.low_u64(),
        config.pool_size as usize,
        config.broadcast_txs_size,
        config.broadcast_txs_interval,
    );
    let mempool = Arc::new(
        MemPoolImpl::new(
            config.pool_size as usize,
            config.timeout_gap,
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

fn init_crypto(
    privkey: &[u8],
    validators: &[ValidatorExtend],
) -> ProtocolResult<Arc<OverlordCrypto>> {
    let bls_priv_key = BlsPrivateKey::try_from(privkey).map_err(MainError::Crypto)?;

    let mut bls_pub_keys = HashMap::new();
    for validator_extend in validators.iter() {
        let address = validator_extend.pub_key.as_bytes();
        let hex_pubkey = validator_extend.bls_pub_key.as_bytes();
        let pub_key = BlsPublicKey::try_from(hex_pubkey.as_ref()).map_err(MainError::Crypto)?;
        bls_pub_keys.insert(address, pub_key);
    }

    // The `common_ref` is a placeholder, use empty string.
    let crypto = OverlordCrypto::new(bls_priv_key, bls_pub_keys, String::new());
    Ok(Arc::new(crypto))
}

async fn get_status_agent(
    storage: &Arc<impl Storage>,
    block: &Block,
    metadata: &Metadata,
) -> ProtocolResult<StatusAgent> {
    let header = &block.header;
    let latest_proof = storage.get_latest_proof(Context::new()).await?;
    let current_consensus_status = CurrentStatus {
        prev_hash:       block.hash(),
        last_number:     header.number,
        max_tx_size:     metadata.consensus_config.max_tx_size.into(),
        tx_num_limit:    metadata.consensus_config.tx_num_limit,
        proof:           latest_proof,
        last_state_root: header.state_root,
    };

    CHAIN_ID.swap(Arc::new(header.chain_id));

    let status_agent = StatusAgent::new(current_consensus_status);
    Ok(status_agent)
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
        if let Err(e) =
            overlord_consensus
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

async fn execute_genesis(
    mut partial_genesis: RichBlock,
    spec: &ChainSpec,
    db_group: &DatabaseGroup,
) -> ProtocolResult<RichBlock> {
    let metadata_0 = spec.params.clone();
    let metadata_1 =
        {
            let mut tmp = metadata_0.clone();
            tmp.epoch = metadata_0.epoch + 1;
            tmp.version.start = metadata_0.version.end + 1;
            tmp.version.end = tmp.version.start + metadata_0.version.end - 1;
            tmp
        };

    let resp = execute_genesis_transactions(
        &partial_genesis,
        db_group,
        &spec.accounts,
        &[metadata_0, metadata_1],
        spec.genesis.generate_hardfork_info(),
    )?;

    partial_genesis.block.header.state_root = resp.state_root;
    partial_genesis.block.header.receipts_root = resp.receipt_root;

    let logs = resp
        .tx_resp
        .iter()
        .map(|r| Bloom::from(BloomInput::Raw(rlp::encode_list(&r.logs).as_ref())))
        .collect::<Vec<_>>();
    let log_bloom = Bloom::from(BloomInput::Raw(rlp::encode_list(&logs).as_ref()));

    partial_genesis.block.header.log_bloom = log_bloom;

    log::info!("The genesis block is executed {:?}", partial_genesis.block);
    log::info!("Response for genesis is {:?}", resp);

    db_group
        .storage()
        .save_block(&partial_genesis, &resp)
        .await?;

    Ok(partial_genesis)
}

fn execute_genesis_transactions(
    rich: &RichBlock,
    db_group: &DatabaseGroup,
    accounts: &[InitialAccount],
    metadata_list: &[Metadata],
    hardfork: HardforkInfoInner,
) -> ProtocolResult<ExecResp> {
    let state_root =
        MPTTrie::new(db_group.trie_db())
            .insert_accounts(accounts)
            .expect("insert accounts")
            .commit()?;
    let mut backend = AxonExecutorApplyAdapter::from_root(
        state_root,
        db_group.trie_db(),
        db_group.storage(),
        Proposal::new_without_state_root(&rich.block.header).into(),
    )?;

    system_contract::init(db_group.inner_db(), &mut backend, metadata_list, hardfork)?;

    let resp = AxonExecutor.exec(&mut backend, &rich.txs, &[]);

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

pub fn set_hardfork_info(
    config: Config,
    hardfork_info: Option<HardforkInfoInner>,
) -> ProtocolResult<()> {
    let path_rocksdb = config.data_path_for_rocksdb();
    if !path_rocksdb.exists() {
        let msg = format!(
            "Data directory {} doesn't exist, please initialize it before run.",
            path_rocksdb.display()
        );
        return Err(MainError::Other(msg).into());
    }

    let rt = RuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()
        .expect("new tokio runtime");

    rt.block_on(async move {
        log::info!("Load databases.");
        let db_group = DatabaseGroup::new(
            &config.rocksdb,
            path_rocksdb,
            false,
            config.executor.triedb_cache_size,
        )?;

        let storage = db_group.storage();
        let trie_db = db_group.trie_db();
        let inner_db = db_group.inner_db();

        let current_block = storage.get_latest_block(Context::new()).await?;
        let current_state_root = current_block.header.state_root;

        // Init system contract DB
        let mut backend = AxonExecutorApplyAdapter::from_root(
            current_block.header.state_root,
            Arc::clone(&trie_db),
            Arc::clone(&storage),
            Proposal::new_without_state_root(&current_block.header).into(),
        )?;

        system_contract::init_system_contract_db(inner_db, &mut backend);

        let metadata_root =
            AxonExecutorReadOnlyAdapter::from_root(
                current_state_root,
                Arc::clone(&trie_db),
                Arc::clone(&storage),
                Proposal::new_without_state_root(&current_block.header).into(),
            )?
            .get_metadata_root();

        let metadata_handle = MetadataHandle::new(metadata_root);
        metadata_handle.init_hardfork(current_block.header.number)?;
        if let Some(proposed_info) = hardfork_info {
            if current_block.header.number >= proposed_info.block_number {
                return Err::<(), protocol::ProtocolError>(
                    MainError::Other(format!(
                        "Hardfork start block number {} less than current number {}",
                        proposed_info.block_number, current_block.header.number
                    ))
                    .into(),
                );
            }

            if proposed_info.flags == H256::zero() {
                return Ok(());
            }

            let hardforks = metadata_handle.hardfork_infos()?;

            if let Some(current_info) = hardforks.inner.last() {
                // if hardfork is already activated, return
                if current_info.flags & proposed_info.flags == proposed_info.flags {
                    log::info!("this feature has been actived");
                    return Ok(());
                }
            }

            storage
                .set_hardfork_proposal(Default::default(), proposed_info)
                .await?;
        }
        Ok(())
    })?;

    Ok(())
}
