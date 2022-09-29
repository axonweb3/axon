use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::H160;

use core_ibc::IBC_HANDLER;

use crate::err;
use crate::precompiles::{precompile_address, PrecompileContract};

#[derive(Default, Clone)]
pub struct IbcHandle;

impl PrecompileContract for IbcHandle {
    const ADDRESS: H160 = precompile_address(0xfe);
    const MIN_GAS: u64 = 1000;

    fn exec_fn(
        input: &[u8],
        _gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
        let res = IBC_HANDLER.get().unwrap().handle(input);

        if res == 0 {
            return Ok(PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                cost:        150000,
                output:      vec![res],
                logs:        vec![],
            });
        }

        err!()
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        unreachable!()
    }
}
