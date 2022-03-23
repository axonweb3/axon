use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use common_crypto::{secp256k1_recover, Secp256k1RecoverableSignature, Signature};
use protocol::types::{Hasher, H160};

use crate::precompiles::{precompile_address, PrecompileContract};

#[derive(Default, Clone)]
pub struct EcRecover;

impl PrecompileContract for EcRecover {
    const ADDRESS: H160 = precompile_address(0x01);
    const MIN_GAS: u64 = 3000;

    fn exec_fn(
        origin_input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
        if let Some(limit) = gas_limit {
            if limit < Self::gas_cost(origin_input) {
                return Err(PrecompileFailure::Error {
                    exit_status: ExitError::OutOfGas,
                });
            }
        }

        let mut input = [0u8; 128];
        let len = origin_input.len().min(128);
        input[..len].copy_from_slice(&origin_input[..len]);

        let sig = match recover_signature(&input) {
            Some(value) => value,
            None => {
                return Err(PrecompileFailure::Error {
                    exit_status: ExitError::InvalidCode,
                })
            }
        };

        if let Ok(s) = Secp256k1RecoverableSignature::try_from(sig.as_slice()) {
            if let Ok(p) = secp256k1_recover(&s.to_bytes(), &input[0..32]) {
                let r = Hasher::digest(&p.serialize_uncompressed());
                let mut recover = vec![0u8; 12];
                recover.append(&mut r.as_bytes().to_vec());

                return Ok(PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    cost:        EcRecover::MIN_GAS,
                    output:      recover,
                    logs:        vec![],
                });
            }
        }

        Err(PrecompileFailure::Error {
            exit_status: ExitError::InvalidCode,
        })
    }

    fn gas_cost(_input: &[u8]) -> u64 {
        Self::MIN_GAS
    }
}

fn recover_signature(input: &[u8]) -> Option<[u8; 65]> {
    let mut ret = [0u8; 65];
    ret[0..64].copy_from_slice(&input[64..128]);

    let tmp = &input[32..64];
    let v = match tmp[31] {
        27 | 28 if tmp[..31] == [0; 31] => tmp[31] - 27,
        _ => {
            return None;
        }
    };
    ret[64] = v;

    Some(ret)
}
