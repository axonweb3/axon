use std::ops::Range;

use bn::{pairing_batch, AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::{H160, U256};

use crate::err;
use crate::precompiles::{precompile_address, PrecompileContract};

#[derive(Default)]
pub struct EcPairing;

impl PrecompileContract for EcPairing {
    const ADDRESS: H160 = precompile_address(0x08);
    const MIN_GAS: u64 = 45_000;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
        if input.is_empty() {
            let mut buf = [0u8; 32];
            U256::one().to_big_endian(&mut buf);

            return Ok(PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                cost:        Self::MIN_GAS,
                output:      buf.to_vec(),
                logs:        Default::default(),
            });
        }

        let gas = Self::gas_cost(input);
        if let Some(limit) = gas_limit {
            if limit < gas {
                return err!();
            }
        }

        let elements = input.len() / Self::GROUP_ARGS_LEN;
        let mut pairs = Vec::with_capacity(elements);

        for i in 0..elements {
            let a_x = Fq::from_slice(&input[Self::index_range(i, 0)])
                .map_err(|_| err!(_, "Invalid X coordinate"))?;
            let a_y = Fq::from_slice(&input[Self::index_range(i, 32)])
                .map_err(|_| err!(_, "Invalid Y coordinate"))?;
            let b_a_y = Fq::from_slice(&input[Self::index_range(i, 64)])
                .map_err(|_| err!(_, "Invalid imaginary coeff X coordinate"))?;
            let b_a_x = Fq::from_slice(&input[Self::index_range(i, 96)])
                .map_err(|_| err!(_, "Invalid imaginary coeff Y coordinate"))?;
            let b_b_y = Fq::from_slice(&input[Self::index_range(i, 128)])
                .map_err(|_| err!(_, "Invalid real coeff X coordinate"))?;
            let b_b_x = Fq::from_slice(&input[Self::index_range(i, 160)])
                .map_err(|_| err!(_, "Invalid real coeff Y coordinate"))?;

            let b_a = Fq2::new(b_a_x, b_a_y);
            let b_b = Fq2::new(b_b_x, b_b_y);

            let a = if a_x.is_zero() && a_y.is_zero() {
                G1::zero()
            } else {
                G1::from(AffineG1::new(a_x, a_y).map_err(|_| err!(_, "Not on curve"))?)
            };

            let b = if b_a.is_zero() && b_b.is_zero() {
                G2::zero()
            } else {
                G2::from(AffineG2::new(b_a, b_b).map_err(|_| err!(_, "Not on curve"))?)
            };

            pairs.push((a, b));
        }

        let mut buf = [0u8; 32];
        if pairing_batch(&pairs) == Gt::one() {
            U256::one().to_big_endian(&mut buf);
        } else {
            U256::zero().to_big_endian(&mut buf);
        }

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost:        Self::MIN_GAS,
            output:      buf.to_vec(),
            logs:        Default::default(),
        })
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let elements = (input.len() / Self::GROUP_ARGS_LEN) as u64;
        elements * Self::GAS_PER_PAIRING + Self::MIN_GAS
    }
}

impl EcPairing {
    const GAS_PER_PAIRING: u64 = 34_000;
    const GROUP_ARGS_LEN: usize = 192;

    fn index_range(i: usize, offset: usize) -> Range<usize> {
        let start = i * Self::GROUP_ARGS_LEN + offset;
        start..(start + 32)
    }
}
