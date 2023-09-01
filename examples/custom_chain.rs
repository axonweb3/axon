#![allow(clippy::diverging_sub_expression)]

use axon::{async_trait, FeeAllocate, FeeInlet, KeyProvider, ValidatorExtend, H160, U256};

#[derive(Default, Clone, Debug)]
struct CustomFeeAllocator;

impl FeeAllocate for CustomFeeAllocator {
    fn allocate(
        &self,
        _block_number: U256,
        _fee_collect: U256,
        _proposer: H160,
        _validators: &[ValidatorExtend],
    ) -> Vec<FeeInlet> {
        // Write your custom fee allocation process below.
        todo!()
    }
}

#[derive(Default, Clone, Debug)]
struct CustomKey;

#[async_trait]
impl KeyProvider for CustomKey {
    type Error = std::io::Error;

    async fn sign_ecdsa_async<T: AsRef<[u8]> + Send>(
        &self,
        _message: T,
    ) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn sign_ecdsa<T: AsRef<[u8]>>(&self, _message: T) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn pubkey(&self) -> Vec<u8> {
        todo!()
    }

    fn verify_ecdsa<P, T, F>(&self, _pubkey: P, _message: T, _signature: F) -> bool
    where
        P: AsRef<[u8]>,
        T: AsRef<[u8]>,
        F: AsRef<[u8]>,
    {
        todo!()
    }
}

fn main() {
    let result = axon::run(CustomFeeAllocator, CustomKey, "0.1.0");
    if let Err(e) = result {
        eprintln!("Error {e}");
        std::process::exit(1);
    }
}
