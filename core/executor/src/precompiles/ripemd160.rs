use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};
use ripemd::Digest;

use protocol::types::H160;

use crate::err;
use crate::precompiles::{precompile_address, PrecompileContract};

#[derive(Default, Clone)]
pub struct Ripemd160;

impl PrecompileContract for Ripemd160 {
    const ADDRESS: H160 = precompile_address(0x03);
    const MIN_GAS: u64 = 600;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
        let gas = Self::gas_cost(input);
        if let Some(limit) = gas_limit {
            if gas > limit {
                return err!();
            }
        }

        let mut ret = [0u8; 32];
        let mut hasher = ripemd::Ripemd160::default();
        hasher.update(input);
        ret[12..].copy_from_slice(&hasher.finalize());

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost:        gas,
            output:      ret.to_vec(),
            logs:        vec![],
        })
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let data_word_size = (input.len() + 31) / 32;
        (data_word_size * 120) as u64 + Self::MIN_GAS
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::precompile_test;

    #[test]
    fn test_ripemd160() {
        precompile_test!(
            Ripemd160,
            "ff",
            "0000000000000000000000002c0c45d3ecab80fe060e5f1d7057cd2f8de5e557",
            720
        );
    }
}
