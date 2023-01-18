use ckb_types::{core::TransactionView, packed, prelude::*};
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};
use rlp::Rlp;
use rlp_derive::{RlpDecodable, RlpEncodable};

use protocol::traits::Interoperation;
use protocol::types::{Bytes, H160, H256};

use core_interoperation::{cycle_to_gas, gas_to_cycle, InteroperationImpl};

use crate::precompiles::{precompile_address, PrecompileContract};
use crate::{err, system_contract::image_cell::DataProvider};

macro_rules! try_rlp {
    ($rlp_: expr, $func: ident, $pos: expr) => {{
        $rlp_.$func($pos).map_err(|e| err!(_, e.to_string()))?
    }};
}

// pub struct Payload {
//     pub inputs:      Vec<CellWithWitness>,
//     pub cell_deps:   Vec<packed::CellDep>,
//     pub header_deps: Vec<packed::Byte32>,
// }

#[derive(RlpEncodable, RlpDecodable, Clone, Debug)]
pub struct CellWithWitness {
    pub tx_hash:      H256,
    pub index:        u32,
    pub witness_type: Option<Bytes>,
    pub witness_lock: Option<Bytes>,
}

#[derive(RlpEncodable, RlpDecodable, Clone, Debug)]
pub struct CellDep {
    pub tx_hash:  H256,
    pub index:    u32,
    pub dep_type: u8,
}

impl From<CellDep> for packed::CellDep {
    fn from(dep: CellDep) -> packed::CellDep {
        packed::CellDepBuilder::default()
            .out_point(
                packed::OutPointBuilder::default()
                    .tx_hash(dep.tx_hash.0.pack())
                    .index(dep.index.pack())
                    .build(),
            )
            .dep_type(packed::Byte::new(dep.dep_type))
            .build()
    }
}

#[derive(Default, Clone)]
pub struct CkbVM;

impl PrecompileContract for CkbVM {
    const ADDRESS: H160 = precompile_address(0xf1);
    const MIN_GAS: u64 = 500;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        if let Some(gas) = gas_limit {
            let res = <InteroperationImpl as Interoperation>::verify_by_ckb_vm(
                Default::default(),
                &DataProvider::default(),
                &mock_transaction(&Rlp::new(input))?,
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

fn mock_transaction(rlp: &Rlp) -> Result<TransactionView, PrecompileFailure> {
    let inputs: Vec<CellWithWitness> = try_rlp!(rlp, list_at, 0);
    let cell_deps: Vec<CellDep> = try_rlp!(rlp, list_at, 1);
    let header_deps: Vec<H256> = try_rlp!(rlp, list_at, 2);

    Ok(build_mock_tx(inputs, cell_deps, header_deps))
}

pub fn build_mock_tx(
    inputs: Vec<CellWithWitness>,
    cell_deps: Vec<CellDep>,
    header_deps: Vec<H256>,
) -> TransactionView {
    TransactionView::new_advanced_builder()
        .inputs(inputs.iter().map(|i| {
            packed::CellInput::new(
                packed::OutPointBuilder::default()
                    .tx_hash(i.tx_hash.0.pack())
                    .index(i.index.pack())
                    .build(),
                5u64,
            )
        }))
        .witnesses(inputs.iter().map(|i| {
            packed::WitnessArgsBuilder::default()
                .input_type(
                    packed::BytesOptBuilder::default()
                        .set(i.witness_type.clone().map(|inner| inner.pack()))
                        .build(),
                )
                .lock(
                    packed::BytesOptBuilder::default()
                        .set(i.witness_type.clone().map(|inner| inner.pack()))
                        .build(),
                )
                .build()
                .as_bytes()
                .pack()
        }))
        .cell_deps(cell_deps.into_iter().map(Into::into))
        .header_deps(header_deps.iter().map(|dep| dep.0.pack()))
        .build()
}
