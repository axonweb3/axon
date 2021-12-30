use std::collections::BTreeMap;
use std::str::FromStr;

use evm::backend::{MemoryAccount, MemoryBackend, MemoryVicinity};

use protocol::codec::ProtocolCodec;
use protocol::traits::Executor;
use protocol::types::{
    ExitReason, ExitSucceed, Public, SignatureComponents, SignedTransaction, Transaction,
    TransactionAction, UnverifiedTransaction, H160, H256, U256,
};

use crate::EvmExecutor;

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

fn gen_tx(sender: H160, addr: H160, data: Vec<u8>) -> SignedTransaction {
    SignedTransaction {
        transaction: UnverifiedTransaction {
            unsigned:  Transaction {
                nonce:                    U256::default(),
                max_priority_fee_per_gas: U256::default(),
                gas_price:                U256::default(),
                gas_limit:                U256::from_str("0x1000000000").unwrap(),
                action:                   TransactionAction::Call(addr),
                value:                    U256::default(),
                data:                     data.into(),
                access_list:              Vec::new(),
            },
            signature: Some(SignatureComponents {
                standard_v: 0,
                r:          H256::default(),
                s:          H256::default(),
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
			code: hex::decode("60e060020a6000350480632839e92814601e57806361047ff414603457005b602a6004356024356047565b8060005260206000f35b603d6004356099565b8060005260206000f35b600082600014605457605e565b8160010190506093565b81600014606957607b565b60756001840360016047565b90506093565b609060018403608c85600186036047565b6047565b90505b92915050565b6000816000148060a95750816001145b60b05760b7565b81905060cf565b60c1600283036099565b60cb600184036099565b0190505b91905056").unwrap(),
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
    let executor = EvmExecutor::new();
    let tx = gen_tx(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
        hex::decode("2839e92800000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001").unwrap()
    );
    let r = executor.inner_exec(&mut backend, tx);
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
    assert_eq!(r.ret, vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 13
    ]);
    assert_eq!(r.remain_gas, 18446744073709518456);
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

    let executor = EvmExecutor::new();

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
        hex::decode(simplestorage_create_code).unwrap(),
    );
    tx.transaction.unsigned.action = TransactionAction::Create;
    let r = executor.inner_exec(&mut backend, tx);
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
    assert!(r.ret.is_empty());
    assert_eq!(r.remain_gas, 18446744073709450374);

    // Thr created contract's address is
    // 0x1334d12e187d9aa97ea520fdd100c5d4f867ade0, you can get the address from
    // the ApplyBackend.

    // let's call SimpleStorage.set(42)
    let tx = gen_tx(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0x1334d12e187d9aa97ea520fdd100c5d4f867ade0").unwrap(),
        hex::decode("60fe47b1000000000000000000000000000000000000000000000000000000000000002a")
            .unwrap(),
    );
    let r = executor.inner_exec(&mut backend, tx);
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Stopped));
    assert!(r.ret.is_empty());
    assert_eq!(r.remain_gas, 18446744073709508106);

    // let's call SimpleStorage.get() by exec
    let tx = gen_tx(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0x1334d12e187d9aa97ea520fdd100c5d4f867ade0").unwrap(),
        hex::decode("6d4ce63c").unwrap(),
    );
    let r = executor.inner_exec(&mut backend, tx);
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
    assert_eq!(r.ret, vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 42
    ]);
    assert_eq!(r.remain_gas, 18446744073709528227);

    // let's call SimpleStorage.get() by call
    let r = executor.call(
        &mut backend,
        H160::from_str("0x1334d12e187d9aa97ea520fdd100c5d4f867ade0").unwrap(),
        hex::decode("6d4ce63c").unwrap(),
    );
    assert_eq!(r.exit_reason, ExitReason::Succeed(ExitSucceed::Returned));
    assert_eq!(r.ret, vec![
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 42
    ]);
}

#[test]
fn test_uniswap() {
    let mut stx: SignedTransaction = UnverifiedTransaction::decode(
        hex::decode("f9048b82053917849502f900849502f90282d8488080b90431608060405234801561001057600080fd5b50336000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555060008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16600073ffffffffffffffffffffffffffffffffffffffff167f342827c97908e5e2f71151c08502a66d44b6f758e3ac2f1de95f02eb95f0a73560405160405180910390a3610356806100db6000396000f3fe608060405234801561001057600080fd5b50600436106100365760003560e01c8063893d20e81461003b578063a6f9dae114610059575b600080fd5b610043610075565b604051610050919061025d565b60405180910390f35b610073600480360381019061006e91906101fe565b61009e565b005b60008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff16905090565b60008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff163373ffffffffffffffffffffffffffffffffffffffff161461012c576040517f08c379a000000000000000000000000000000000000000000000000000000000815260040161012390610278565b60405180910390fd5b8073ffffffffffffffffffffffffffffffffffffffff1660008054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff167f342827c97908e5e2f71151c08502a66d44b6f758e3ac2f1de95f02eb95f0a73560405160405180910390a3806000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555050565b6000813590506101f881610309565b92915050565b600060208284031215610214576102136102db565b5b6000610222848285016101e9565b91505092915050565b610234816102a9565b82525050565b6000610247601383610298565b9150610252826102e0565b602082019050919050565b6000602082019050610272600083018461022b565b92915050565b600060208201905081810360008301526102918161023a565b9050919050565b600082825260208201905092915050565b60006102b4826102bb565b9050919050565b600073ffffffffffffffffffffffffffffffffffffffff82169050919050565b600080fd5b7f43616c6c6572206973206e6f74206f776e657200000000000000000000000000600082015250565b610312816102a9565b811461031d57600080fd5b5056fea26469706673582212208f65d9c920a5e7c951f66594688d8b1bb886420d647e05d3e5c9ec7eeab019cd64736f6c63430008070033c080a03e0133186f198a17429635dcfa58b8a744fffadbf3fe125acf28aa421e3a5ba8a0653f08fa21f0f88c272abde7ed60a38c08b543a7628b53c8a39e4465c0c110ca").unwrap()
    ).unwrap().try_into().unwrap();
    let sender = stx.sender;

    println!("{:?}", stx.transaction);

    stx.transaction.unsigned.value = U256::zero();
    let mut state = BTreeMap::new();
    state.insert(sender, MemoryAccount {
        nonce:   U256::zero(),
        balance: U256::max_value(),
        storage: BTreeMap::new(),
        code:    Vec::new(),
    });

    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, state);
    let executor = EvmExecutor::new();
    let res = executor.inner_exec(&mut backend, stx);

    println!("{:?}", res);

    assert!(res.exit_reason.is_succeed())
}
