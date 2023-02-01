#![allow(clippy::uninlined_format_args)]

#[cfg(test)]
mod tests;
mod utils;

use std::error::Error;

use ckb_script::TransactionScriptsVerifier;
use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::{cell::CellProvider, Cycle, TransactionView};
use ckb_types::packed;
use ckb_vm::machine::{asm::AsmCoreMachine, DefaultMachineBuilder, SupportMachine, VERSION1};
use ckb_vm::{Error as VMError, ISA_B, ISA_IMC, ISA_MOP};

use protocol::traits::{Context, Interoperation};
use protocol::types::{Bytes, VMResp};
use protocol::{Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

use crate::utils::resolve_transaction;

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
        TransactionScriptsVerifier::new(
            &resolve_transaction(&data_loader, mocked_tx)?,
            &data_loader,
        )
        .verify(max_cycles)
        .map_err(|e| InteroperationError::Ckb(e).into())
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
