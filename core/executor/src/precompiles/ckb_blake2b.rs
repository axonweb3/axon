use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::{ckb_blake2b_256, types::H160};

use crate::err;
use crate::precompiles::{axon_precompile_address, PrecompileContract};

#[derive(Default, Clone)]
pub struct CkbBlake2b;

impl PrecompileContract for CkbBlake2b {
    const ADDRESS: H160 = axon_precompile_address(0x06);
    const MIN_GAS: u64 = 60;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        let gas = Self::gas_cost(input);
        if let Some(limit) = gas_limit {
            if gas > limit {
                return err!();
            }
        }

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output:      ckb_blake2b_256(input).to_vec(),
            },
            gas,
        ))
    }

    /// Estimate the gas cost = MIN_GAS + dynamic_gas
    ///                       = MIN_GAS + 12 * data_word_size
    fn gas_cost(input: &[u8]) -> u64 {
        let data_word_size = (input.len() + 31) / 32;
        (data_word_size * 12) as u64 + Self::MIN_GAS
    }
}
