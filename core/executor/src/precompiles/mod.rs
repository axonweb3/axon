mod blake2_f;
mod call_ckb_vm;
mod ckb_blake2b;
mod ckb_mbt_verify;
mod ec_add;
mod ec_mul;
mod ec_pairing;
mod ecrecover;
mod get_cell;
mod get_header;
mod identity;
mod modexp;
mod ripemd160;
mod rsa;
mod secp256r1;
mod sha256;

#[cfg(test)]
mod tests;

use std::collections::BTreeMap;

use bn::{AffineG1, Fq, Fr, Group, G1};
use evm::executor::stack::{PrecompileFailure, PrecompileFn, PrecompileOutput};
use evm::{Context, ExitError};

use protocol::types::H160;

use crate::precompiles::{
    blake2_f::Blake2F, call_ckb_vm::CallCkbVM, ckb_blake2b::CkbBlake2b, ckb_mbt_verify::CMBTVerify,
    ec_add::EcAdd, ec_mul::EcMul, ec_pairing::EcPairing, ecrecover::EcRecover,
    get_header::GetHeader, identity::Identity, modexp::ModExp, ripemd160::Ripemd160,
    sha256::Sha256,
};

#[macro_export]
macro_rules! err {
    () => {
        Err(PrecompileFailure::Error {
            exit_status: ExitError::OutOfGas,
        })
    };

    ($msg: expr) => {
        Err(PrecompileFailure::Error {
            exit_status: ExitError::Other($msg.into()),
        })
    };

    (_, $msg: expr) => {
        PrecompileFailure::Error {
            exit_status: ExitError::Other($msg.into()),
        }
    };
}

macro_rules! precompiles {
    () => { BTreeMap::new() };

    ($($contract: ident),+) => {{
        let mut set = BTreeMap::new();
        $(
            set.insert($contract::ADDRESS, $contract::exec_fn as PrecompileFn);
        )*
        set
    }};
}

trait PrecompileContract {
    const ADDRESS: H160;
    const MIN_GAS: u64;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        context: &Context,
        is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure>;

    fn gas_cost(input: &[u8]) -> u64;
}

const fn eip_precompile_address(addr: u8) -> H160 {
    H160([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, addr,
    ])
}

const fn axon_precompile_address(addr: u8) -> H160 {
    H160([
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x01, addr,
    ])
}

pub fn build_precompile_set() -> BTreeMap<H160, PrecompileFn> {
    precompiles!(
        EcRecover, Sha256, Ripemd160, Identity, ModExp, EcAdd, EcMul, EcPairing, Blake2F,
        CallCkbVM, CkbBlake2b, CMBTVerify, GetHeader
    )
}

pub(crate) fn read_point(input: &[u8], start: usize) -> Result<G1, PrecompileFailure> {
    if input.len() < start + 64 {
        return err!("Invalid input length");
    }

    let px =
        Fq::from_slice(&input[start..(start + 32)]).map_err(|_| err!(_, "Invalid X coordinate"))?;

    let py = Fq::from_slice(&input[(start + 32)..(start + 64)])
        .map_err(|_| err!(_, "Invalid Y coordinate"))?;

    let ret = if px == Fq::zero() && py == Fq::zero() {
        G1::zero()
    } else {
        AffineG1::new(px, py)
            .map_err(|_| err!(_, "Invalid curve point"))?
            .into()
    };

    Ok(ret)
}

pub(crate) fn read_fr(input: &[u8], start: usize) -> Result<Fr, PrecompileFailure> {
    if input.len() < start + 32 {
        return err!("Invalid input length");
    }

    Fr::from_slice(&input[start..(start + 32)]).map_err(|_| err!(_, "Invalid field element"))
}
