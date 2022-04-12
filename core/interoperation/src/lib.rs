#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use arc_swap::ArcSwap;
use ckb_vm::machine::asm::{AsmCoreMachine, AsmMachine};
use ckb_vm::machine::{DefaultMachineBuilder, SupportMachine, VERSION1};
use ckb_vm::{Error as VMError, ISA_B, ISA_IMC, ISA_MOP};

use protocol::traits::{Context, Interoperation};
use protocol::types::{Bytes, SignedTransaction, VMResp, H256};
use protocol::{Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

lazy_static::lazy_static! {
    static ref DISPATCHER: ArcSwap<ProgramDispatcher> = ArcSwap::from_pointee(ProgramDispatcher::default());
    static ref CODE_HASH_MAP: ArcSwap<HashMap<String, H256>> = ArcSwap::from_pointee(HashMap::new());
}

const ISA: u8 = ISA_IMC | ISA_B | ISA_MOP;

const GAS_TO_CYCLE_COEF: u64 = 6_000;

pub const fn gas_to_cycle(gas: u64) -> u64 {
    gas * GAS_TO_CYCLE_COEF
}

pub const fn cycle_to_gas(cycle: u64) -> u64 {
    cycle / GAS_TO_CYCLE_COEF
}

pub enum SignatureType {
    Secp256k1,
    Ed25519,
}

impl TryFrom<u8> for SignatureType {
    type Error = InteroperationError;

    fn try_from(s: u8) -> Result<Self, Self::Error> {
        match s {
            0 | 1 => Ok(SignatureType::Secp256k1),
            2 => Ok(SignatureType::Ed25519),
            _ => Err(InteroperationError::InvalidSignatureType(s)),
        }
    }
}

#[derive(Default, Clone)]
pub struct InteroperationImpl;

impl Interoperation for InteroperationImpl {
    fn verify_external_signature(
        &self,
        _ctx: Context,
        tx: SignedTransaction,
    ) -> ProtocolResult<()> {
        let _sig_type = SignatureType::try_from(
            tx.transaction
                .signature
                .ok_or(InteroperationError::MissingSignature)?
                .standard_v,
        )?;

        Ok(())
    }

    fn call_ckb_vm(
        &self,
        _ctx: Context,
        code_hash: H256,
        args: &[Bytes],
        max_cycles: u64,
    ) -> ProtocolResult<VMResp> {
        let core =
            DefaultMachineBuilder::new(AsmCoreMachine::new(ISA, VERSION1, max_cycles)).build();
        let program = DISPATCHER.load().get_program(&code_hash)?;

        #[cfg(not(target_arch = "aarch64"))]
        let aot_code = unsafe { Some(&*Arc::as_ptr(&program.aot)) };

        #[cfg(target_arch = "aarch64")]
        let aot_code = None;

        let mut vm = AsmMachine::new(core, aot_code);
        let _ = vm
            .load_program(&program.code, args)
            .map_err(InteroperationError::CkbVM)?;

        Ok(VMResp {
            exit_code: vm.run().map_err(InteroperationError::CkbVM)?,
            cycles:    vm.machine.cycles(),
        })
    }
}

impl InteroperationImpl {
    pub fn new(
        code_hash_map: HashMap<String, H256>,
        program_map: HashMap<H256, Bytes>,
    ) -> ProtocolResult<Self> {
        init_dispatcher(program_map)?;
        init_code_hashes_map(code_hash_map);
        Ok(InteroperationImpl::default())
    }
}

fn init_dispatcher(program_map: HashMap<H256, Bytes>) -> ProtocolResult<()> {
    DISPATCHER.swap(Arc::new(ProgramDispatcher::new(program_map)?));
    Ok(())
}

fn init_code_hashes_map(hashes: HashMap<String, H256>) {
    CODE_HASH_MAP.swap(Arc::new(hashes));
}

pub fn get_crypto_code_hash(crypto: &str) -> ProtocolResult<H256> {
    let crypto = String::from(crypto);
    if let Some(code_hash) = CODE_HASH_MAP.load().get(&crypto) {
        Ok(*code_hash)
    } else {
        Err(InteroperationError::GetCryptoCodeHash(crypto).into())
    }
}

#[derive(Default)]
struct ProgramDispatcher(HashMap<H256, Program>);

impl ProgramDispatcher {
    #[cfg(not(target_arch = "aarch64"))]
    fn new(program_map: HashMap<H256, Bytes>) -> ProtocolResult<Self> {
        let mut inner = HashMap::with_capacity(program_map.len());

        for (code_hash, code) in program_map.into_iter() {
            let aot_code =
                ckb_vm::machine::aot::AotCompilingMachine::load(&code, None, ISA, VERSION1)
                    .and_then(|mut m| m.compile())
                    .map_err(InteroperationError::CkbVM)?;
            inner.insert(code_hash, Program::new(code, aot_code));
        }

        Ok(ProgramDispatcher(inner))
    }

    #[cfg(target_arch = "aarch64")]
    fn new(program_map: HashMap<H256, Bytes>) -> ProtocolResult<Self> {
        Ok(ProgramDispatcher(
            program_map
                .into_iter()
                .map(|kv| (kv.0, Program::new(kv.1)))
                .collect(),
        ))
    }

    fn get_program(&self, code_hash: &H256) -> ProtocolResult<Program> {
        self.0
            .get(code_hash)
            .cloned()
            .ok_or_else(|| InteroperationError::GetProgram(*code_hash).into())
    }
}

#[derive(Clone)]
struct Program {
    code: Bytes,
    #[cfg(not(target_arch = "aarch64"))]
    aot:  Arc<ckb_vm::machine::asm::AotCode>,
}

impl Program {
    #[cfg(not(target_arch = "aarch64"))]
    fn new(code: Bytes, aot: ckb_vm::machine::asm::AotCode) -> Self {
        Program {
            code,
            aot: Arc::new(aot),
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn new(code: Bytes) -> Self {
        Program { code }
    }
}

#[derive(Debug, Display)]
pub enum InteroperationError {
    #[display(fmt = "Invalid signature type value v is {:?}", _0)]
    InvalidSignatureType(u8),

    #[display(fmt = "Transaction missing signature")]
    MissingSignature,

    #[display(fmt = "Cannot get program of code hash {:?}", _0)]
    GetProgram(H256),

    #[display(fmt = "CKB VM run failed {:?}", _0)]
    CkbVM(VMError),

    #[display(fmt = "Unsupported ckb crypto primitive {:?}", _0)]
    GetCryptoCodeHash(String),
}

impl Error for InteroperationError {}

impl From<InteroperationError> for ProtocolError {
    fn from(error: InteroperationError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Interoperation, Box::new(error))
    }
}
