#[cfg(test)]
#[allow(dead_code)]
mod tests;
pub mod utils;

use std::{error::Error, sync::Arc};

use ckb_chain_spec::consensus::Consensus;
use ckb_script::{TransactionScriptsVerifier, TxVerifyEnv};
use ckb_traits::CellDataProvider;
use ckb_types::core::{Cycle, HeaderBuilder, TransactionView};
use ckb_types::{h256, packed, prelude::Pack, H256};
use ckb_vm::machine::{asm::AsmCoreMachine, DefaultMachineBuilder, SupportMachine, VERSION1};
use ckb_vm::{Error as VMError, ISA_B, ISA_IMC, ISA_MOP};

use protocol::traits::{CkbDataProvider, Context, Interoperation};
use protocol::types::{Bytes, CellDep, CellWithData, OutPoint, VMResp};
use protocol::{Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

use crate::utils::resolve_transaction;

const ISA: u8 = ISA_IMC | ISA_B | ISA_MOP;
const GAS_TO_CYCLE_COEF: u64 = 6_000;

// The following information is from CKB block [10976708](https://explorer.nervos.org/block/10976708)
// which is CKB2023 disabled.
const CKB2023_DISABLED_NUMBER: u64 = 10_976_708;
const CKB2023_DISABLED_EPOCH: u64 = 0x53c007f0020c8;
const CKB2023_DISABLED_PARENT_HASH: H256 =
    h256!("0xeed08d487af4723238ef82b4e4388e2ad2dc8dc28e4e759f5d891eeab79ff3ff");

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
        data_cell_dep: CellDep,
        args: &[Bytes],
        max_cycles: u64,
    ) -> ProtocolResult<VMResp> {
        let data_cell_dep: packed::CellDep = (&data_cell_dep).into();
        let program = data_loader
            .get_cell_data(&data_cell_dep.out_point())
            .ok_or_else(|| InteroperationError::GetProgram((&data_cell_dep.out_point()).into()))?;
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

    /// Todo: After CKB2023 is enabled, a hardfork is needed to support the new
    /// VM version and syscalls.
    fn verify_by_ckb_vm<DL: CkbDataProvider + Sync + Send + 'static>(
        _ctx: Context,
        data_loader: DL,
        mocked_tx: &TransactionView,
        dummy_input: Option<CellWithData>,
        max_cycles: u64,
    ) -> ProtocolResult<Cycle> {
        let rtx = Arc::new(resolve_transaction(&data_loader, mocked_tx, dummy_input)?);
        log::debug!("[mempool]: Verify by ckb vm tx {:?}", rtx);

        // The consensus and tx_env arguments are used for judge if the VM version2 and
        // syscalls3 are enabled. Due to only the hardfork field in consensus and the
        // epoch field in tx_env is used, the provided arguments only need to fill these
        // fields correctly.
        let (ckb_spec, ckb2023_disabled_env) = {
            let consensus = Arc::new(Consensus::default());
            let header = HeaderBuilder::default()
                .number(CKB2023_DISABLED_NUMBER.pack())
                .epoch(CKB2023_DISABLED_EPOCH.pack())
                .parent_hash(CKB2023_DISABLED_PARENT_HASH.pack())
                .build();
            let tx_env = Arc::new(TxVerifyEnv::new_commit(&header));
            (consensus, tx_env)
        };

        TransactionScriptsVerifier::new(rtx, data_loader, ckb_spec, ckb2023_disabled_env)
            .verify(max_cycles)
            .map_err(|e| InteroperationError::Ckb(e).into())
    }
}

#[derive(Debug, Display)]
pub enum InteroperationError {
    #[display(fmt = "Transaction missing signature")]
    MissingSignature,

    #[display(fmt = "Cannot get program of out point {:?}", _0)]
    GetProgram(OutPoint),

    #[display(fmt = "CKB VM verify script failed {:?}", _0)]
    Ckb(ckb_error::Error),

    #[display(fmt = "CKB VM call failed {:?}", _0)]
    CkbVM(VMError),

    #[display(fmt = "Unsupported blockchain id {:?}", _0)]
    GetBlockchainCodeHash(u8),

    #[display(fmt = "Get unknown cell by out point {:?}", _0)]
    GetUnknownCell(OutPoint),

    #[display(fmt = "Invalid dep group {:?}", _0)]
    InvalidDepGroup(String),

    #[display(fmt = "Invalid dummy input")]
    InvalidDummyInput,
}

impl Error for InteroperationError {}

impl From<InteroperationError> for ProtocolError {
    fn from(error: InteroperationError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Interoperation, Box::new(error))
    }
}
