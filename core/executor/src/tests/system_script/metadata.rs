use std::{collections::BTreeMap, str::FromStr};

use ethers::abi::AbiEncode;

use core_db::RocksAdapter;
use protocol::types::{MemoryBackend, SignedTransaction, H160, U256};

use crate::{
    system_contract::{
        init_system_contract_db,
        metadata::{
            metadata_abi::{self, ConsensusConfig, Metadata, MetadataVersion, ValidatorExtend},
            MetadataContract, MetadataStore,
        },
        SystemContract, METADATA_CONTRACT_ADDRESS,
    },
    tests::{gen_tx, gen_vicinity},
    CURRENT_METADATA_ROOT,
};

static ROCKSDB_PATH: &str = "./free-space/system-contract/metadata";

#[test]
fn test_write_functions() {
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());

    let executor = MetadataContract::default();
    let inner_db = RocksAdapter::new(ROCKSDB_PATH, Default::default())
        .unwrap()
        .inner_db();
    init_system_contract_db(inner_db, &mut backend);

    test_init(&mut backend, &executor);

    let mut vicinity = gen_vicinity();
    vicinity.block_number += U256::one();
    backend = MemoryBackend::new(&vicinity, BTreeMap::new());

    test_second(&mut backend, &executor);
    test_validator(&mut backend, &executor);

    test_update_consensus_config(&mut backend, &executor);
}

fn test_init<'a>(backend: &mut MemoryBackend<'a>, executor: &MetadataContract<MemoryBackend<'a>>) {
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    let tx = prepare_tx_1(&addr);
    let r = executor.exec_(backend, &tx);
    assert!(r.exit_reason.is_succeed());
}

fn test_second<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &MetadataContract<MemoryBackend<'a>>,
) {
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();

    // this transaction will fail because the epoch is not incremental
    // epoch should be 1 but is passed as 0
    let tx = prepare_tx_2(&addr);
    let r = executor.exec_(backend, &tx);
    assert!(r.exit_reason.is_revert());

    // this transaction will fail because the version is not incremental
    let tx = prepare_tx_3(&addr);
    let r = executor.exec_(backend, &tx);
    assert!(r.exit_reason.is_revert());

    let tx = prepare_tx_4(&addr);
    let r = executor.exec_(backend, &tx);
    assert!(r.exit_reason.is_succeed());

    // this transaction will fail because the epoch is not incremental
    // epoch should be 2 but is passed as 3
    let tx = prepare_tx_5(&addr);
    let r = executor.exec_(backend, &tx);
    assert!(r.exit_reason.is_revert());
}

fn test_validator<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &MetadataContract<MemoryBackend<'a>>,
) {
    let addr = H160::from_str("0x0000000000000000000000000000000000000000").unwrap();

    // this transaction will fail because the sender is not in validator list
    let tx = prepare_tx_4(&addr);
    let r = executor.exec_(backend, &tx);
    assert!(r.exit_reason.is_revert());
}

fn prepare_tx_1(addr: &H160) -> SignedTransaction {
    let data = metadata_abi::AppendMetadataCall {
        metadata: prepare_metadata(),
    };

    gen_tx(*addr, METADATA_CONTRACT_ADDRESS, 1000, data.encode())
}

fn prepare_tx_2(addr: &H160) -> SignedTransaction {
    let mut data = metadata_abi::AppendMetadataCall {
        metadata: prepare_metadata(),
    };
    data.metadata.version.start = 101;
    data.metadata.version.end = 200;

    gen_tx(*addr, METADATA_CONTRACT_ADDRESS, 1000, data.encode())
}

fn prepare_tx_3(add: &H160) -> SignedTransaction {
    let mut data = metadata_abi::AppendMetadataCall {
        metadata: prepare_metadata(),
    };
    data.metadata.epoch = 1;
    data.metadata.version.start = 1;
    data.metadata.version.end = 100;

    gen_tx(*add, METADATA_CONTRACT_ADDRESS, 1000, data.encode())
}

fn prepare_tx_4(addr: &H160) -> SignedTransaction {
    let mut data = metadata_abi::AppendMetadataCall {
        metadata: prepare_metadata(),
    };
    data.metadata.epoch = 1;
    data.metadata.version.start = 101;
    data.metadata.version.end = 200;

    gen_tx(*addr, METADATA_CONTRACT_ADDRESS, 1000, data.encode())
}

fn prepare_tx_5(addr: &H160) -> SignedTransaction {
    let mut data = metadata_abi::AppendMetadataCall {
        metadata: prepare_metadata(),
    };
    data.metadata.epoch = 3;
    data.metadata.version.start = 201;
    data.metadata.version.end = 300;

    gen_tx(*addr, METADATA_CONTRACT_ADDRESS, 1000, data.encode())
}

fn prepare_tx_with_consensus_config(addr: &H160, interval: u64) -> SignedTransaction {
    let data = metadata_abi::UpdateConsensusConfigCall {
        config: {
            let mut config = prepare_metadata().consensus_config;
            config.interval = interval;
            config
        },
    };

    gen_tx(*addr, METADATA_CONTRACT_ADDRESS, 1000, data.encode())
}

// change consensus interval test
fn test_update_consensus_config<'a>(
    backend: &mut MemoryBackend<'a>,
    executor: &MetadataContract<MemoryBackend<'a>>,
) {
    let interval = 10;
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    let tx = prepare_tx_with_consensus_config(&addr, interval);

    let r = executor.exec_(backend, &tx);
    assert!(r.exit_reason.is_succeed());

    let root = CURRENT_METADATA_ROOT.with(|r| *r.borrow());

    let store = MetadataStore::new(root).unwrap();

    let current_config = store.get_metadata(1).unwrap().consensus_config;

    assert_eq!(current_config.interval, interval)
}

fn prepare_metadata() -> Metadata {
    Metadata {
        version:          MetadataVersion {
            start: 1u64,
            end:   100u64,
        },
        epoch:            0,
        verifier_list:    vec![prepare_validator()],
        propose_counter:  vec![],
        consensus_config: ConsensusConfig {
            gas_limit:          1u64,
            interval:           0u64,
            propose_ratio:      1u64,
            prevote_ratio:      1u64,
            precommit_ratio:    1u64,
            brake_ratio:        1u64,
            tx_num_limit:       1u64,
            max_tx_size:        1u64,
            max_contract_limit: 0x6000u64,
        },
    }
}

fn prepare_validator() -> ValidatorExtend {
    ValidatorExtend {
        bls_pub_key:    [1u8; 32].into(),
        pub_key:        [1u8; 32].into(),
        address:        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        propose_weight: 1u32,
        vote_weight:    1u32,
    }
}

// #[tokio::test]
// async fn update_consensus_config() {
//     let config:
// crate::system_contract::metadata::metadata_abi::ConsensusConfig =
// ConsensusConfig {         gas_limit:          0x3e7fffffc18,
//         interval:           0xbb8,
//         propose_ratio:      0xf,
//         prevote_ratio:      0xa,
//         precommit_ratio:    0xa,
//         brake_ratio:        0xa,
//         tx_num_limit:       0x4e20,
//         max_tx_size:        0x186a0000,
//         max_contract_limit: 0x8000u64,
//     }
//     .into();

//     let tx_data =
//         crate::system_contract::metadata::metadata_abi::UpdateConsensusConfigCall { config }
//             .encode();

//     send_eth_tx("http://127.0.0.1:8000", tx_data, METADATA_CONTRACT_ADDRESS).await
// }
// use ethers::prelude::*;
// use ethers::signers::{LocalWallet, Signer};
// use ethers::types::transaction::eip2718::TypedTransaction::Legacy;
// use ethers::types::{Address, TransactionRequest};

// const ADDRESS: &str = "0x8ab0CF264DF99D83525e9E11c7e4db01558AE1b1";
// const PRIVATE_KEY: &str =
// "37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d"; pub async
// fn send_eth_tx(axon_url: &str, data: Vec<u8>, to: Address) {     let provider
// = Provider::<Http>::try_from(axon_url).unwrap();

//     let from: Address = ADDRESS.parse().unwrap();
//     let nonce = provider.get_transaction_count(from, None).await.unwrap();

//     let transaction_request = TransactionRequest::new()
//         .chain_id(0x41786f6e)
//         .to(to)
//         .data(data)
//         .from(from)
//         .gas_price(1)
//         .gas(21000)
//         .nonce(nonce);

//     let wallet = LocalWallet::from_str(PRIVATE_KEY).expect(
//         "failed to create
// wallet",
//     );
//     let tx = Legacy(transaction_request);
//     let signature: Signature = wallet.sign_transaction(&tx).await.unwrap();

//     provider
//         .send_raw_transaction(tx.rlp_signed(&signature))
//         .await
//         .unwrap()
//         .await
//         .unwrap()
//         .expect("failed to send eth tx");
// }
