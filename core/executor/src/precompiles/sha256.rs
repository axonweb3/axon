use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};
use sha2::Digest;

use protocol::types::H160;

use crate::err;
use crate::precompiles::{precompile_address, PrecompileContract};

#[derive(Default, Clone)]
pub struct Sha256;

impl PrecompileContract for Sha256 {
    const ADDRESS: H160 = precompile_address(0x02);
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

        let mut hasher = sha2::Sha256::default();
        hasher.update(input);

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output:      hasher.finalize().to_vec(),
            },
            gas,
        ))
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let data_word_size = (input.len() + 31) / 32;
        (data_word_size * 12) as u64 + Self::MIN_GAS
    }
}
