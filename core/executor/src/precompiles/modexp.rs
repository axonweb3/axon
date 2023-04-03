use std::ops::BitAnd;

use az::SaturatingAs;
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};
use rug::ops::Pow;
use rug::{integer::Order, Integer};

use protocol::types::H160;

use crate::err;
use crate::precompiles::{eip_precompile_address, PrecompileContract};

#[derive(Default)]
pub struct ModExp;

impl PrecompileContract for ModExp {
    const ADDRESS: H160 = eip_precompile_address(0x05);
    const MIN_GAS: u64 = 200;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<(PrecompileOutput, u64), PrecompileFailure> {
        let large_number = LargeNumber::parse(input)?;

        let gas = Self::gas_cost(input);
        if let Some(limit) = gas_limit {
            if limit < gas {
                return err!();
            }
        }

        let m_size = large_number.m_size;
        let mut res = large_number.calc()?.to_digits::<u8>(Order::MsfBe);
        let res_len = res.len();

        if res_len > m_size {
            return err!("Exec failed");
        }

        if res_len < m_size {
            let mut ret = vec![0u8; m_size - res_len];
            ret.append(&mut res);

            return Ok((
                PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output:      ret,
                },
                gas,
            ));
        }

        Ok((
            PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                output:      res,
            },
            gas,
        ))
    }

    fn gas_cost(input: &[u8]) -> u64 {
        match LargeNumber::parse(input) {
            Ok(large_number) => {
                let dynamic_gas =
                    large_number.multiplication_complexity() * large_number.iterator_count() / 3u64;

                dynamic_gas
                    .max(Integer::from(Self::MIN_GAS))
                    .saturating_as()
            }
            Err(_) => u64::MAX,
        }
    }
}

fn get_data(data: &[u8], mut start: usize, size: usize) -> Result<Integer, PrecompileFailure> {
    let len = data.len();

    if start > len {
        start = len;
    }

    let mut end = start.wrapping_add(size);
    if end > len {
        end = len;
    }

    let mut padded = if start < end {
        data[start..end].to_vec()
    } else {
        Vec::new()
    };

    padded
        .try_reserve_exact(size)
        .map_err(|_| PrecompileFailure::Error {
            exit_status: ExitError::StackOverflow,
        })?;

    padded.extend(std::iter::repeat(0).take(size - (end - start)));

    Ok(Integer::from_digits(&padded, Order::MsfBe))
}

struct LargeNumber {
    b_size:   usize,
    e_size:   usize,
    m_size:   usize,
    base:     Integer,
    exponent: Integer,
    modulo:   Integer,
}

impl LargeNumber {
    fn parse(input: &[u8]) -> Result<Self, PrecompileFailure> {
        let base_size = get_data(input, 0, 32)?.saturating_as::<usize>();
        let exponent_size = get_data(input, 32, 32)?.saturating_as::<usize>();
        let modulo_size = get_data(input, 64, 32)?.saturating_as::<usize>();
        let data = if input.len() > 96 {
            &input[96..]
        } else {
            &input[0..0]
        };

        Ok(LargeNumber {
            b_size:   base_size,
            e_size:   exponent_size,
            m_size:   modulo_size,
            base:     get_data(data, 0, base_size)?,
            exponent: get_data(data, base_size, exponent_size)?,
            modulo:   get_data(data, base_size.wrapping_add(exponent_size), modulo_size)?,
        })
    }

    fn multiplication_complexity(&self) -> Integer {
        Integer::from((self.b_size.max(self.m_size) + 7) / 8).pow(2)
    }

    fn iterator_count(&self) -> u64 {
        let iter_count = if self.e_size <= 32 && self.exponent == Integer::ZERO {
            0
        } else if self.e_size <= 32 {
            (self.exponent.significant_bits() - 1) as usize
        } else {
            let bytes: [u8; 32] = [0xFF; 32];
            let max_256_bit_uint = Integer::from_digits(&bytes, Order::MsfBe);
            (8 * (self.e_size - 32))
                + ((self.exponent.clone().bitand(max_256_bit_uint)).significant_bits() - 1) as usize
        };

        iter_count.max(1) as u64
    }

    fn calc(self) -> Result<Integer, PrecompileFailure> {
        if self.b_size == 0 && self.m_size == 0 {
            return Ok(Integer::ZERO);
        }

        // https://github.com/ethereum/go-ethereum/blob/a03490c6b2ff0e1d9a1274afdbe087a695d533eb/core/vm/contracts.go#L385
        if self.modulo == Integer::ZERO {
            return Ok(Integer::ZERO);
        } else if Integer::from(self.base.abs_ref()) == 1 {
            return Ok(self.base % self.modulo);
        }

        self.base
            .pow_mod(&self.exponent, &self.modulo)
            .map_err(|_| err!(_, "Overflow"))
    }
}
