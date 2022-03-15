use super::*;
use crate::{SystemExecutor, NATIVE_TOKEN_ISSUE_ADDRESS};

fn mock_data(direction: u8, address: H160) -> Vec<u8> {
    let mut ret = vec![direction];
    ret.extend_from_slice(&address.0);
    ret
}

#[test]
fn test_issue_token() {
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, BTreeMap::new());
    let executor = SystemExecutor::default();
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    let data = mock_data(0, addr);
    let tx = gen_tx(addr, NATIVE_TOKEN_ISSUE_ADDRESS, 1000, data);

    let r = executor.inner_exec(&mut backend, tx);
    assert!(r.exit_reason.is_succeed());
    assert_eq!(r.ret, rlp::encode(&U256::from(1000)).to_vec());

    let account = backend.state().get(&addr).unwrap();
    assert_eq!(account.balance, U256::from(1000u64));
    assert_eq!(account.nonce, U256::from(1u64));
}

#[test]
fn test_burn_token() {
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    let mut state = BTreeMap::new();
    state.insert(
		addr,
		MemoryAccount {
			nonce: U256::one(),
			balance: U256::from(2000u64),
			storage: BTreeMap::new(),
			code: Vec::new(),
		}
	);
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, state);
    let executor = SystemExecutor::default();
    let data = mock_data(1, addr);
    let tx = gen_tx(addr, NATIVE_TOKEN_ISSUE_ADDRESS, 1000, data);

    let r = executor.inner_exec(&mut backend, tx);
    assert!(r.exit_reason.is_succeed());
    assert_eq!(r.ret, rlp::encode(&U256::from(1000)).to_vec());

    let account = backend.state().get(&addr).unwrap();
    assert_eq!(account.balance, U256::from(1000u64));
    assert_eq!(account.nonce, U256::from(2u64));
}

#[test]
fn test_burn_token_failed() {
    let addr = H160::from_str("0xf000000000000000000000000000000000000000").unwrap();
    let mut state = BTreeMap::new();
    state.insert(
		addr,
		MemoryAccount {
			nonce: U256::one(),
			balance: U256::from(200u64),
			storage: BTreeMap::new(),
			code: Vec::new(),
		}
	);
    let vicinity = gen_vicinity();
    let mut backend = MemoryBackend::new(&vicinity, state);
    let executor = SystemExecutor::default();
    let data = mock_data(1, addr);
    let tx = gen_tx(addr, NATIVE_TOKEN_ISSUE_ADDRESS, 1000, data);

    let r = executor.inner_exec(&mut backend, tx);
    assert!(r.exit_reason.is_revert());
    assert!(r.ret.is_empty());

    let account = backend.state().get(&addr).unwrap();
	println!("{:?}", account);
    assert_eq!(account.balance, U256::from(200u64));
    assert_eq!(account.nonce, U256::from(2u64));
}
