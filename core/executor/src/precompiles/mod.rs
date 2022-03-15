mod blake2_f;
mod ecrecover;
mod modexp;
mod rsa;
mod secp256r1;
mod sha256;

use std::collections::BTreeMap;

use evm::executor::stack::PrecompileFn;

use protocol::types::H160;

trait PrecompileContract {
    fn address(&self) -> H160;
    fn exec_fn(&self) -> PrecompileFn;
}

pub fn build_precompile_set() -> BTreeMap<H160, PrecompileFn> {
    let ret = BTreeMap::new();
    ret
}
