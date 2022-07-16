mod system_script;

use std::collections::BTreeMap;
use std::str::FromStr;

use evm::backend::{MemoryAccount, MemoryBackend, MemoryVicinity};
use evm::Config;

use protocol::types::{
    Bytes, Eip1559Transaction, ExitError, ExitReason, ExitSucceed, Public, SignatureComponents,
    SignedTransaction, TransactionAction, UnsignedTransaction, UnverifiedTransaction, H160, H256,
    MAX_BLOCK_GAS_LIMIT, U256,
};
use protocol::{codec::hex_decode, traits::Executor};

use crate::{precompiles::build_precompile_set, vm::EvmExecutor, AxonExecutor};

fn gen_vicinity() -> MemoryVicinity {
    MemoryVicinity {
        gas_price:              U256::zero(),
        origin:                 H160::default(),
        block_hashes:           Vec::new(),
        block_number:           Default::default(),
        block_coinbase:         Default::default(),
        block_timestamp:        Default::default(),
        block_difficulty:       Default::default(),
        block_gas_limit:        Default::default(),
        chain_id:               U256::one(),
        block_base_fee_per_gas: U256::zero(),
    }
}

fn gen_tx(sender: H160, addr: H160, value: u64, data: Vec<u8>) -> SignedTransaction {
    SignedTransaction {
        transaction: UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Eip1559(Eip1559Transaction {
                nonce:                    U256::default(),
                max_priority_fee_per_gas: U256::default(),
                gas_price:                U256::default(),
                gas_limit:                U256::from_str("0x1000000000").unwrap(),
                action:                   TransactionAction::Call(addr),
                value:                    value.into(),
                data:                     data.into(),
                access_list:              Vec::new(),
            }),
            signature: Some(SignatureComponents {
                standard_v: 0,
                r:          Bytes::default(),
                s:          Bytes::default(),
            }),
            chain_id:  0u64,
            hash:      H256::default(),
        },
        sender,
        public: Some(Public::default()),
    }
}

#[test]
fn test_ackermann31() {
    let mut state = BTreeMap::new();
    state.insert(
		H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
		MemoryAccount {
			nonce: U256::one(),
			balance: U256::max_value(),
			storage: BTreeMap::new(),
			code: hex_decode("60e060020a6000350480632839e92814601e57806361047ff414603457005b602a6004356024356047565b8060005260206000f35b603d6004356099565b8060005260206000f35b600082600014605457605e565b8160010190506093565b81600014606957607b565b60756001840360016047565b90506093565b609060018403608c85600186036047565b6047565b90505b92915050565b6000816000148060a95750816001145b60b05760b7565b81905060cf565b60c1600283036099565b60cb600184036099565b0190505b91905056").unwrap(),
		}
	);
    state.insert(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        MemoryAccount {
            nonce:   U256::one(),
            balance: U256::max_value(),
            storage: BTreeMap::new(),
            code:    Vec::new(),
        },
    );

    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, state);
    let executor = EvmExecutor::default();
    let tx = gen_tx(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
        0,
        hex_decode("2839e92800000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001").unwrap()
    );
    let config = Config::london();
    let precompiles = build_precompile_set();
    let r = executor.inner_exec(&mut backend, &config, MAX_BLOCK_GAS_LIMIT, &precompiles, tx);

    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
    assert_eq!(r.ret, vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 13
    ]);
    assert_eq!(r.remain_gas, 29966841);
}

#[test]
fn test_simplestorage() {
    let mut state = BTreeMap::new();
    state.insert(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        MemoryAccount {
            nonce:   U256::one(),
            balance: U256::max_value(),
            storage: BTreeMap::new(),
            code:    Vec::new(),
        },
    );
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, state);

    let executor = EvmExecutor::default();
    let config = Config::london();
    let precompiles = build_precompile_set();

    // pragma solidity ^0.4.24;
    //
    // contract SimpleStorage {
    //     uint storedData;
    //
    //     function set(uint x) public {
    //         storedData = x;
    //     }
    //
    //     function get() view public returns (uint) {
    //         return storedData;
    //     }
    // }
    //
    // simplestorage_create_code created from above solidity
    let simplestorage_create_code = "608060405234801561001057600080fd5b5060df8061001f6000396000f3006080604052600436106049576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806360fe47b114604e5780636d4ce63c146078575b600080fd5b348015605957600080fd5b5060766004803603810190808035906020019092919050505060a0565b005b348015608357600080fd5b50608a60aa565b6040518082815260200191505060405180910390f35b8060008190555050565b600080549050905600a165627a7a7230582099c66a25d59f0aa78f7ebc40748fa1d1fbc335d8d780f284841b30e0365acd960029";
    let mut tx = gen_tx(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
        0,
        hex_decode(simplestorage_create_code).unwrap(),
    );
    tx.transaction
        .unsigned
        .set_action(TransactionAction::Create);
    let r = executor.inner_exec(&mut backend, &config, MAX_BLOCK_GAS_LIMIT, &precompiles, tx);
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
    assert!(r.ret.is_empty());
    assert_eq!(r.remain_gas, 29898759);

    // Thr created contract's address is
    // 0xc15d2ba57d126e6603240e89437efd419ce329d2, you can get the address by
    // `println!("{:?}", backend.state().keys());`

    // let's call SimpleStorage.set(42)
    let tx = gen_tx(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0xc15d2ba57d126e6603240e89437efd419ce329d2").unwrap(),
        0,
        hex_decode("60fe47b1000000000000000000000000000000000000000000000000000000000000002a")
            .unwrap(),
    );
    let r = executor.inner_exec(&mut backend, &config, MAX_BLOCK_GAS_LIMIT, &precompiles, tx);
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Stopped));
    assert!(r.ret.is_empty());
    assert_eq!(r.remain_gas, 29956491);

    // let's call SimpleStorage.get() by exec
    let tx = gen_tx(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0xc15d2ba57d126e6603240e89437efd419ce329d2").unwrap(),
        0,
        hex_decode("6d4ce63c").unwrap(),
    );
    let r = executor.inner_exec(&mut backend, &config, MAX_BLOCK_GAS_LIMIT, &precompiles, tx);
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
    // assert_eq!(r.ret, vec![
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // 0, 0, 0, 0, 0, 0,     0, 42
    // ]);
    assert_eq!(r.remain_gas, 29976612);

    // let's call SimpleStorage.get() by call
    let executor = AxonExecutor::default();
    let r = executor.call(
        &mut backend,
        u64::MAX,
        None,
        Some(H160::from_str("0xc15d2ba57d126e6603240e89437efd419ce329d2").unwrap()),
        U256::default(),
        hex_decode("6d4ce63c").unwrap(),
    );
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
    // assert_eq!(r.ret, vec![
    //     0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // 0, 0, 0, 0, 0, 0, 0,     0, 42
    // ]);
}

#[test]
fn test_out_of_gas() {
    let to_address = H160::from_str("0x1000000000000000000000000000000000000000").unwrap();
    let from_address = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();

    let mut state = BTreeMap::new();
    state.insert(
		to_address,
		MemoryAccount {
			nonce: U256::one(),
			balance: U256::max_value(),
			storage: BTreeMap::new(),
			code: hex_decode("60e060020a6000350480632839e92814601e57806361047ff414603457005b602a6004356024356047565b8060005260206000f35b603d6004356099565b8060005260206000f35b600082600014605457605e565b8160010190506093565b81600014606957607b565b60756001840360016047565b90506093565b609060018403608c85600186036047565b6047565b90505b92915050565b6000816000148060a95750816001145b60b05760b7565b81905060cf565b60c1600283036099565b60cb600184036099565b0190505b91905056").unwrap(),
		}
	);
    state.insert(from_address, MemoryAccount {
        nonce:   U256::one(),
        balance: U256::max_value(),
        storage: BTreeMap::new(),
        code:    Vec::new(),
    });

    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, state);
    let executor = EvmExecutor::default();

    let tx = SignedTransaction {
        transaction: UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Eip1559(Eip1559Transaction {
                nonce:                    U256::default(),
                max_priority_fee_per_gas: U256::default(),
                gas_price:                U256::default(),
                gas_limit:                U256::from(10),
                action:                   TransactionAction::Call(to_address),
                value:                    U256::zero(),
                data:                     hex_decode("2839e92800000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001").unwrap().into(),
                access_list:              Vec::new(),
            }),
            signature: Some(SignatureComponents {
                standard_v: 0,
                r:          Bytes::default(),
                s:          Bytes::default(),
            }),
            chain_id:  0u64,
            hash:      H256::default(),
        },
        sender: from_address,
        public: Some(Public::default()),
    };

    let config = Config::london();
    let precompiles = build_precompile_set();
    let r = executor.inner_exec(&mut backend, &config, 10, &precompiles, tx);

    assert_eq!(r.exit_reason, ExitReason::Error(ExitError::OutOfGas));
    assert_eq!(
        backend.state().get(&from_address).unwrap().nonce,
        U256::from(2)
    );
}
