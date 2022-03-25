use evm::Context;
use rand::random;
use sha2::Digest;

use protocol::codec::hex_decode;

use crate::precompiles::{Identity, PrecompileContract, Ripemd160, Sha256};

macro_rules! test_precompile {
    ($ty: ident, $input: expr, $output: expr, $expect_gas_cost: expr) => {
        let gas_cost = <$ty as PrecompileContract>::gas_cost($input);
        let resp = <$ty as PrecompileContract>::exec_fn($input, None, &mock_context(), false);
        assert!(resp.is_ok());
        assert_eq!(resp.unwrap().output, $output);
        assert_eq!(gas_cost, $expect_gas_cost);
    };

    ($ty: ident, $input: expr, $gas_limit: expr, $expect_gas_cost: expr, $err: expr) => {
        let gas_cost = <$ty as PrecompileContract>::gas_cost($input);
        let resp =
            <$ty as PrecompileContract>::exec_fn($input, Some($gas_limit), &mock_context(), false);
        assert!(resp.is_err());
        assert_eq!(gas_cost, expect_gas_cost);
    };
}

fn mock_context() -> Context {
    Context {
        address:        Default::default(),
        caller:         Default::default(),
        apparent_value: Default::default(),
    }
}

fn rand_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|_| random::<u8>()).collect()
}

#[test]
fn test_sha256() {
    let input = rand_bytes(100);
    let mut hasher = sha2::Sha256::default();
    hasher.update(&input);
    let output = hasher.finalize().to_vec();

    test_precompile!(Sha256, &input, output, 108);
}

#[test]
fn test_ripemd160() {
    let input = &[0xff];
    let output = hex_decode("2c0c45d3ecab80fe060e5f1d7057cd2f8de5e557").unwrap();
    test_precompile!(Ripemd160, input, output, 720);
}

#[test]
fn test_identity() {
	let data = rand_bytes(16);
	test_precompile!(Identity, &data, data, 18);
}