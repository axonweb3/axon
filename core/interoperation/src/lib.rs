#![allow(clippy::uninlined_format_args)]

#[cfg(test)]
mod tests;

use std::error::Error;

use ckb_script::TransactionScriptsVerifier;
use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::{cell::ResolvedTransaction, Cycle};
use ckb_types::packed;
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
    Ethereum,
    Other(u8),
}

impl From<u8> for BlockchainType {
    fn from(s: u8) -> Self {
        match s {
            0 | 1 => BlockchainType::Ethereum,
            _ => BlockchainType::Other(s),
        }
    }
}

#[derive(Default, Clone)]
pub struct InteroperationImpl;

impl Interoperation for InteroperationImpl {
    fn call_ckb_vm<'a, DL: CellDataProvider>(
        _ctx: Context,
        data_loader: &'a DL,
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

    fn verify_by_ckb_vm<'a, DL: CellDataProvider + HeaderProvider>(
        _ctx: Context,
        data_loader: &'a DL,
        mocked_tx: &'a ResolvedTransaction,
        max_cycles: u64,
    ) -> ProtocolResult<Cycle> {
        let cycles = TransactionScriptsVerifier::new(mocked_tx, data_loader)
            .verify(max_cycles)
            .map_err(InteroperationError::Ckb)?;
        Ok(cycles)
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
}

impl Error for InteroperationError {}

impl From<InteroperationError> for ProtocolError {
    fn from(error: InteroperationError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Interoperation, Box::new(error))
    }
}
