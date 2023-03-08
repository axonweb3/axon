use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};
use rlp::Rlp;

use protocol::traits::Interoperation;
use protocol::types::{CellDep, OutPoint, SignatureR, SignatureS, Witness, H160, H256};

use core_interoperation::{cycle_to_gas, gas_to_cycle, InteroperationImpl};

use crate::precompiles::{axon_precompile_address, PrecompileContract};
use crate::{err, system_contract::DataProvider};

macro_rules! try_rlp {
    ($rlp_: expr, $func: ident, $pos: expr) => {{
        $rlp_.$func($pos).map_err(|e| err!(_, e.to_string()))?
    }};
}

#[derive(Default, Clone)]
pub struct CkbVM;

impl PrecompileContract for CkbVM {
    const ADDRESS: H160 = axon_precompile_address(0x01);
    const MIN_GAS: u64 = 500;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        let rlp = Rlp::new(input);
        let cell_deps: Vec<CellDep> = try_rlp!(rlp, list_at, 0);
        let header_deps: Vec<H256> = try_rlp!(rlp, list_at, 1);
        let inputs: Vec<OutPoint> = try_rlp!(rlp, list_at, 2);
        let witnesses: Vec<Witness> = try_rlp!(rlp, list_at, 3);

        if let Some(gas) = gas_limit {
            let res = InteroperationImpl::verify_by_ckb_vm(
                Default::default(),
                &DataProvider::default(),
                &InteroperationImpl::dummy_transaction(
                    SignatureR::new_by_ref(cell_deps, header_deps, inputs, Default::default()),
                    SignatureS::new(witnesses),
                ),
                None,
                gas_to_cycle(gas),
            )
            .map_err(|e| err!(_, e.to_string()))?;

            return Ok((
                PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output:      0i8.to_le_bytes().to_vec(),
                },
                cycle_to_gas(res).max(Self::MIN_GAS),
            ));
        }

        err!()
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        unreachable!()
    }
}
