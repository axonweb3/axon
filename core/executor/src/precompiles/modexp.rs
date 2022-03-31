use std::ops::BitAnd;

use az::UnwrappedAs;
use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};
use rug::ops::Pow;
use rug::{integer::Order, Integer};

use protocol::types::H160;

use crate::err;
use crate::precompiles::{precompile_address, PrecompileContract};

#[derive(Default)]
pub struct ModExp;

impl PrecompileContract for ModExp {
    const ADDRESS: H160 = precompile_address(0x05);
    const MIN_GAS: u64 = 200;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
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

            return Ok(PrecompileOutput {
                exit_status: ExitSucceed::Returned,
                cost:        gas,
                output:      ret,
                logs:        vec![],
            });
        }

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost:        gas,
            output:      res,
            logs:        vec![],
        })
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let large_number = LargeNumber::parse(input).unwrap();
        let dynamic_gas =
            large_number.multiplication_complexity() * large_number.iterator_count() / 3u64;

        dynamic_gas.max(Integer::from(Self::MIN_GAS)).unwrapped_as()
    }
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
    const MAX_NUM_SIZE: usize = 1024;

    fn parse(input: &[u8]) -> Result<Self, PrecompileFailure> {
        let input_len = input.len();
        if input_len < 96 {
            return err!("Input length must be at least 96");
        };

        let base_size = Self::parse_big_uint(&input[0..32], true)?.unwrapped_as::<usize>();
        let exponent_size = Self::parse_big_uint(&input[32..64], true)?.unwrapped_as::<usize>();
        let modulo_size = Self::parse_big_uint(&input[64..96], true)?.unwrapped_as::<usize>();

        let total_len = base_size + exponent_size + modulo_size + 96;
        if input_len < total_len {
            return err!("Insufficient input len");
        }

        let e_start = 96 + base_size;
        let m_start = e_start + exponent_size;

        Ok(LargeNumber {
            b_size:   base_size,
            e_size:   exponent_size,
            m_size:   modulo_size,
            base:     Self::parse_big_uint(&input[96..e_start], false)?,
            exponent: Self::parse_big_uint(&input[e_start..m_start], false)?,
            modulo:   Self::parse_big_uint(&input[m_start..], false)?,
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

        self.base
            .pow_mod(&self.exponent, &self.modulo)
            .map_err(|_| err!(_, "Overflow"))
    }

    fn parse_big_uint(input: &[u8], is_parse_size: bool) -> Result<Integer, PrecompileFailure> {
        let max_size_big = Integer::from(Self::MAX_NUM_SIZE);
        let res = Integer::from_digits(input, Order::MsfBe);

        if is_parse_size && res > max_size_big {
            return err!("The big size must be at most 1024");
        }

        Ok(res)
    }
}
