mod blake2_f;
mod ecrecover;
mod identity;
mod modexp;
mod ripemd160;
mod rsa;
mod secp256r1;
mod sha256;
#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use evm::executor::stack::{PrecompileFailure, PrecompileFn, PrecompileOutput};
use evm::Context;

use protocol::types::H160;

use crate::precompiles::{
    ecrecover::EcRecover, identity::Identity, ripemd160::Ripemd160, sha256::Sha256,
};

macro_rules! precompiles {
    () => { BTreeMap::new() };

    ($($contract: ident),+) => {{
        let mut set = BTreeMap::new();
        $(
            set.insert($contract::ADDRESS, $contract::exec_fn as PrecompileFn);
        )*
        set
    }};
}

trait PrecompileContract {
    const ADDRESS: H160;
    const MIN_GAS: u64;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        context: &Context,
        is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure>;

    fn gas_cost(input: &[u8]) -> u64;
}

const fn precompile_address(addr: u8) -> H160 {
    H160([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, addr,
    ])
}

pub fn build_precompile_set() -> BTreeMap<H160, PrecompileFn> {
    precompiles!(EcRecover, Sha256, Ripemd160, Identity)
}
