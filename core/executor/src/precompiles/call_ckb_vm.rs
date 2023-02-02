use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};
use rlp::Rlp;

use protocol::traits::Interoperation;
use protocol::types::{CellDep, H160, H256};

use core_interoperation::{cycle_to_gas, gas_to_cycle, InteroperationImpl};

use crate::precompiles::{precompile_address, PrecompileContract};
use crate::{err, system_contract::image_cell::DataProvider};

macro_rules! try_rlp {
    ($rlp_: expr, $func: ident, $pos: expr) => {{
        $rlp_.$func($pos).map_err(|e| err!(_, e.to_string()))?
    }};
}

// pub struct Payload {
//     pub tx_hash:  packed::Byte32,
//     pub index:    packed::Uint32,
//     pub dep_type: packed::Uint8,
//     pub arguments: Vec<Bytes>,
// }

#[derive(Default, Clone)]
pub struct CkbVM;

impl PrecompileContract for CkbVM {
    const ADDRESS: H160 = precompile_address(0xf2);
    const MIN_GAS: u64 = 500;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        if let Some(gas) = gas_limit {
            let rlp = Rlp::new(input);
            let res = <InteroperationImpl as Interoperation>::call_ckb_vm(
                Default::default(),
                &DataProvider::default(),
                get_cell_dep(&rlp)?,
                &try_rlp!(rlp, list_at, 3),
                gas_to_cycle(gas),
            )
            .map_err(|e| err!(_, e.to_string()))?;

            return Ok((
                PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output:      res.exit_code.to_le_bytes().to_vec(),
                },
                cycle_to_gas(res.cycles).max(Self::MIN_GAS),
            ));
        }

        err!()
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        unreachable!()
    }
}

fn get_cell_dep(rlp: &Rlp) -> Result<CellDep, PrecompileFailure> {
    let tx_hash: H256 = try_rlp!(rlp, val_at, 0);
    let index: u32 = try_rlp!(rlp, val_at, 1);
    let dep_type: u8 = try_rlp!(rlp, val_at, 2);

    Ok(CellDep {
        tx_hash,
        index,
        dep_type,
    })
}
