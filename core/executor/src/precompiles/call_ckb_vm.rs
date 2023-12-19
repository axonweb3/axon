use ethers::contract::{EthAbiCodec, EthAbiType};
use ethers::{abi::AbiDecode, core::types::Bytes as EthBytes};
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::traits::Interoperation;
use protocol::types::{Bytes, H160};

use core_interoperation::{cycle_to_gas, gas_to_cycle, InteroperationImpl};

use crate::precompiles::{axon_precompile_address, PrecompileContract};
use crate::system_contract::{image_cell::image_cell_abi::OutPoint, DataProvider};
use crate::{err, CURRENT_HEADER_CELL_ROOT};

#[derive(Default, Clone)]
pub struct CallCkbVM;

impl PrecompileContract for CallCkbVM {
    const ADDRESS: H160 = axon_precompile_address(0x04);
    const MIN_GAS: u64 = 500;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        if let Some(gas) = gas_limit {
            let (cell_dep, args) = parse_input(input)?;
            let res = <InteroperationImpl as Interoperation>::call_ckb_vm(
                Default::default(),
                &DataProvider::new(CURRENT_HEADER_CELL_ROOT.with(|r| *r.borrow())),
                cell_dep.into(),
                &args,
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

fn parse_input(input: &[u8]) -> Result<(CellDep, Vec<Bytes>), PrecompileFailure> {
    let payload =
        <(CallCkbVmPayload,) as AbiDecode>::decode(input).map_err(|_| err!(_, "decode input"))?;

    Ok((
        payload.0.cell,
        payload.0.inputs.into_iter().map(|i| i.0).collect(),
    ))
}

#[derive(EthAbiType, EthAbiCodec, Default, Clone, Debug, PartialEq, Eq)]
pub struct CallCkbVmPayload {
    pub cell:   CellDep,
    pub inputs: Vec<EthBytes>,
}

#[derive(EthAbiType, EthAbiCodec, Clone, Default, Debug, PartialEq, Eq)]
pub struct CellDep {
    pub out_point: OutPoint,
    pub dep_type:  u8,
}

impl From<CellDep> for protocol::types::CellDep {
    fn from(v: CellDep) -> Self {
        Self {
            tx_hash:  v.out_point.tx_hash.into(),
            index:    v.out_point.index,
            dep_type: v.dep_type,
        }
    }
}
