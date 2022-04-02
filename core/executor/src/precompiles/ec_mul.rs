use bn::AffineG1;
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::H160;

use crate::err;
use crate::precompiles::{precompile_address, read_fr, read_point, PrecompileContract};

#[derive(Default)]
pub struct EcMul;

impl PrecompileContract for EcMul {
    const ADDRESS: H160 = precompile_address(0x07);
    const MIN_GAS: u64 = 6_000;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
        let gas = Self::gas_cost(input);
        if let Some(limit) = gas_limit {
            if limit < gas {
                return err!();
            }
        }

        let p = read_point(input, 0)?;
        let fr = read_fr(input, 64)?;

        let mut res = [0u8; 64];
        if let Some(sum) = AffineG1::from_jacobian(p * fr) {
            sum.x()
                .to_big_endian(&mut res[0..32])
                .map_err(|_| err!(_, "Invalid sum X"))?;
            sum.y()
                .to_big_endian(&mut res[32..64])
                .map_err(|_| err!(_, "Invalid sum Y"))?;
        }

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost:        gas,
            output:      res.to_vec(),
            logs:        Default::default(),
        })
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        Self::MIN_GAS
    }
}
