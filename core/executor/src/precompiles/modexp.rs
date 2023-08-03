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
        let gas = Self::gas_cost(input);
        if let Some(limit) = gas_limit {
            if limit < gas {
                return err!();
            }
        }

        let base_size = get_data(input, 0, 32).saturating_as::<usize>();
        let modulo_size = get_data(input, 64, 32).saturating_as::<usize>();

        // Handle a special case when both the base and mod length is zero
        if base_size == 0 && modulo_size == 0 {
            return Ok((
                PrecompileOutput {
                    exit_status: ExitSucceed::Returned,
                    output:      Vec::new(),
                },
                gas,
            ));
        }

        let large_number = LargeNumber::parse(input, base_size, modulo_size)?;

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
        let base_size = get_data(input, 0, 32);
        let modulo_size = get_data(input, 64, 32);

        // multiplication_complexity always zero
        if base_size == 0 && modulo_size == 0 {
            return Self::MIN_GAS;
        }

        let exponent_size = get_data(input, 32, 32);

        let data = if input.len() > 96 {
            &input[96..]
        } else {
            &input[0..0]
        };

        let exponent = if exponent_size > 32 {
            get_data(data, base_size.clone().saturating_as::<usize>(), 32)
        } else {
            get_data(
                data,
                base_size.clone().saturating_as::<usize>(),
                exponent_size.clone().saturating_as::<usize>(),
            )
        };

        let multiplication_complexity = multiplication_complexity(base_size, modulo_size);

        let iterator_count = iterator_count(exponent_size, exponent);

        let dynamic_gas = multiplication_complexity * iterator_count / 3u64;
        dynamic_gas
            .max(Integer::from(Self::MIN_GAS))
            .saturating_as::<u64>()
    }
}

fn get_data(data: &[u8], mut start: usize, size: usize) -> Integer {
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

    // may panic here when memory doesn't enough
    padded.reserve_exact(size);

    padded.extend(std::iter::repeat(0).take(size - (end.saturating_sub(start))));

    Integer::from_digits(&padded, Order::MsfBe)
}

struct LargeNumber {
    m_size:   usize,
    base:     Integer,
    exponent: Integer,
    modulo:   Integer,
}

impl LargeNumber {
    fn parse(
        input: &[u8],
        base_size: usize,
        modulo_size: usize,
    ) -> Result<Self, PrecompileFailure> {
        let exponent_size = get_data(input, 32, 32).saturating_as::<usize>();

        let data = if input.len() > 96 {
            &input[96..]
        } else {
            &input[0..0]
        };

        Ok(LargeNumber {
            m_size:   modulo_size,
            base:     get_data(data, 0, base_size),
            exponent: get_data(data, base_size, exponent_size),
            modulo:   get_data(data, base_size.wrapping_add(exponent_size), modulo_size),
        })
    }

    fn calc(self) -> Result<Integer, PrecompileFailure> {
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

fn multiplication_complexity(b_size: Integer, m_size: Integer) -> Integer {
    let a = b_size.max(m_size);
    let a: Integer = a + 7;
    let a: Integer = a / 8;
    a.pow(2)
}

fn iterator_count(e_size: Integer, exponent: Integer) -> u64 {
    let iter_count = if e_size <= 32 && exponent == Integer::ZERO {
        0
    } else if e_size <= 32 {
        (exponent.significant_bits() - 1) as usize
    } else {
        let bytes: [u8; 32] = [0xFF; 32];
        let max_256_bit_uint = Integer::from_digits(&bytes, Order::MsfBe);
        let a: Integer = 8 * (e_size - 32);
        a.saturating_as::<usize>()
            + ((exponent.bitand(max_256_bit_uint))
                .significant_bits()
                .saturating_sub(1)) as usize
    };

    iter_count.max(1) as u64
}
