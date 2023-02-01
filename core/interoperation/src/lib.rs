#![allow(clippy::uninlined_format_args)]

#[cfg(test)]
mod tests;

use std::error::Error;

use ckb_script::TransactionScriptsVerifier;
use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::cell::{CellMeta, CellProvider, CellStatus, ResolvedTransaction};
use ckb_types::core::{Cycle, DepType, TransactionView};
use ckb_types::{packed, prelude::Entity};
use ckb_vm::machine::{asm::AsmCoreMachine, DefaultMachineBuilder, SupportMachine, VERSION1};
use ckb_vm::{Error as VMError, ISA_B, ISA_IMC, ISA_MOP};

use protocol::traits::{Context, Interoperation};
use protocol::types::{Bytes, VMResp};
use protocol::{Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

const ISA: u8 = ISA_IMC | ISA_B | ISA_MOP;
const GAS_TO_CYCLE_COEF: u64 = 6_000;

pub const fn gas_to_cycle(gas: u64) -> u64 {
    gas * GAS_TO_CYCLE_COEF
}

pub const fn cycle_to_gas(cycle: u64) -> u64 {
    cycle / GAS_TO_CYCLE_COEF
}

pub enum BlockchainType {
    BTC,
    Ada,
}

impl TryFrom<u8> for BlockchainType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BlockchainType::BTC),
            1 => Ok(BlockchainType::Ada),
            _ => Err("Unsupported blockchain type".to_string()),
        }
    }
}

#[derive(Default, Clone)]
pub struct InteroperationImpl;

impl Interoperation for InteroperationImpl {
    fn call_ckb_vm<DL: CellDataProvider>(
        _ctx: Context,
        data_loader: &DL,
        data_cell_dep: packed::CellDep,
        args: &[Bytes],
        max_cycles: u64,
    ) -> ProtocolResult<VMResp> {
        let program = data_loader
            .get_cell_data(&data_cell_dep.out_point())
            .ok_or_else(|| InteroperationError::GetProgram(data_cell_dep.out_point()))?;
        let mut vm = ckb_vm::machine::asm::AsmMachine::new(
            DefaultMachineBuilder::new(AsmCoreMachine::new(ISA, VERSION1, max_cycles)).build(),
        );
        let _ = vm
            .load_program(&program, args)
            .map_err(InteroperationError::CkbVM)?;

        Ok(VMResp {
            exit_code: vm.run().map_err(InteroperationError::CkbVM)?,
            cycles:    vm.machine.cycles(),
        })
    }

    fn verify_by_ckb_vm<DL: CellProvider + CellDataProvider + HeaderProvider>(
        _ctx: Context,
        data_loader: DL,
        mocked_tx: &TransactionView,
        max_cycles: u64,
    ) -> ProtocolResult<Cycle> {
        let cycles = TransactionScriptsVerifier::new(
            &resolve_transaction(&data_loader, mocked_tx)?,
            &data_loader,
        )
        .verify(max_cycles)
        .map_err(InteroperationError::Ckb)?;
        Ok(cycles)
    }
}

fn resolve_transaction<CL: CellProvider>(
    cell_loader: &CL,
    tx: &TransactionView,
) -> ProtocolResult<ResolvedTransaction> {
    let resolve_cell = |out_point: &packed::OutPoint| -> ProtocolResult<CellMeta> {
        match cell_loader.cell(out_point, true) {
            CellStatus::Live(meta) => Ok(meta),
            _ => Err(InteroperationError::GetUnknownCell.into()),
        }
    };

    let (mut resolved_inputs, mut resolved_cell_deps, mut resolved_dep_groups) = (
        Vec::with_capacity(tx.inputs().len()),
        Vec::with_capacity(tx.cell_deps().len()),
        Vec::with_capacity(tx.cell_deps().len()),
    );

    for out_point in tx.input_pts_iter() {
        resolved_inputs.push(resolve_cell(&out_point)?);
    }

    for cell_dep in tx.cell_deps_iter() {
        if cell_dep.dep_type() == DepType::DepGroup.into() {
            let outpoint = cell_dep.out_point();
            let dep_group = resolve_cell(&outpoint)?;
            let data = dep_group.mem_cell_data.as_ref().unwrap();
            let sub_out_points =
                parse_dep_group_data(data).map_err(InteroperationError::InvalidDepGroup)?;

            for sub_out_point in sub_out_points.into_iter() {
                resolved_cell_deps.push(resolve_cell(&sub_out_point)?);
            }
            resolved_dep_groups.push(dep_group);
        } else {
            resolved_cell_deps.push(resolve_cell(&cell_dep.out_point())?);
        }
    }

    Ok(ResolvedTransaction {
        transaction: tx.clone(),
        resolved_cell_deps,
        resolved_inputs,
        resolved_dep_groups,
    })
}

pub fn parse_dep_group_data(slice: &[u8]) -> Result<packed::OutPointVec, String> {
    if slice.is_empty() {
        Err("data is empty".to_owned())
    } else {
        match packed::OutPointVec::from_slice(slice) {
            Ok(v) => {
                if v.is_empty() {
                    Err("dep group is empty".to_owned())
                } else {
                    Ok(v)
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

#[derive(Debug, Display)]
pub enum InteroperationError {
    #[display(fmt = "Transaction missing signature")]
    MissingSignature,

    #[display(fmt = "Cannot get program of out point {:?}", _0)]
    GetProgram(packed::OutPoint),

    #[display(fmt = "CKB VM verify script failed {:?}", _0)]
    Ckb(ckb_error::Error),

    #[display(fmt = "CKB VM call failed {:?}", _0)]
    CkbVM(VMError),

    #[display(fmt = "Unsupported blockchain id {:?}", _0)]
    GetBlockchainCodeHash(u8),

    #[display(fmt = "Get unknown cell")]
    GetUnknownCell,

    #[display(fmt = "Invalid dep group {:?}", _0)]
    InvalidDepGroup(String),
}

impl Error for InteroperationError {}

impl From<InteroperationError> for ProtocolError {
    fn from(error: InteroperationError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Interoperation, Box::new(error))
    }
}
