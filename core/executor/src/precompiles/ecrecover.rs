use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use common_crypto::{secp256k1_recover, Secp256k1RecoverableSignature, Signature};
use protocol::types::{Hasher, H160};

use crate::err;
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
        let gas = Self::gas_cost(origin_input);
        if let Some(limit) = gas_limit {
            if limit < gas {
                return err!();
            }
        }

        let mut input = [0u8; 128];
        let len = origin_input.len().min(128);
        input[..len].copy_from_slice(&origin_input[..len]);

        let sig = match recover_signature(&input) {
            Some(value) => value,
            None => {
                return err!("Invalid signature");
            }
        };

        if let Ok(s) = Secp256k1RecoverableSignature::try_from(sig.as_slice()) {
            if let Ok(p) = secp256k1_recover(&input[0..32], &s.to_bytes()) {
                let r = Hasher::digest(&p.serialize_uncompressed()[1..65]);
                let mut recover = [0u8; 32];
                recover[12..].copy_from_slice(&r.as_bytes()[12..]);

                return Ok(PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    cost:        gas,
                    output:      recover.to_vec(),
                    logs:        vec![],
                });
            }
        }

        err!("Verify signature failed")
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
