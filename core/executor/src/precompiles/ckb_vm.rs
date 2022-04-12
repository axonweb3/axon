use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};
use rlp::Rlp;

use core_interoperation::{cycle_to_gas, gas_to_cycle, InteroperationImpl};
use protocol::{traits::Interoperation, types::H160};

use crate::err;
use crate::precompiles::{precompile_address, PrecompileContract};

macro_rules! try_rlp {
    ($rlp_: expr, $func: ident, $pos: expr) => {{
        $rlp_.$func($pos).map_err(|e| err!(_, e.to_string()))?
    }};
}

#[derive(Default, Clone)]
pub struct CkbVM;

impl PrecompileContract for CkbVM {
    const ADDRESS: H160 = precompile_address(0xff);
    const MIN_GAS: u64 = 500;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
        if let Some(gas) = gas_limit {
            let rlp = Rlp::new(input);
            let res = InteroperationImpl::default()
                .call_ckb_vm(
                    Default::default(),
                    try_rlp!(rlp, val_at, 0),
                    &try_rlp!(rlp, list_at, 1),
                    gas_to_cycle(gas),
                )
                .map_err(|e| err!(_, e.to_string()))?;

            return Ok(PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                cost:        cycle_to_gas(res.cycles).max(Self::MIN_GAS),
                output:      res.exit_code.to_le_bytes().to_vec(),
                logs:        vec![],
            });
        }

        err!()
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        unreachable!()
    }
}
