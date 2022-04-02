use evm::executor::stack::{PrecompileFailure, PrecompileOutput};
use evm::{Context, ExitError, ExitSucceed};

use protocol::types::H160;

use crate::err;
use crate::precompiles::{precompile_address, PrecompileContract};

#[derive(Default, Clone)]
pub struct Blake2F;

impl PrecompileContract for Blake2F {
    const ADDRESS: H160 = precompile_address(0x09);
    const MIN_GAS: u64 = 60;

    fn exec_fn(
        input: &[u8],
        gas_limit: Option<u64>,
        _context: &Context,
        _is_static: bool,
    ) -> Result<PrecompileOutput, PrecompileFailure> {
        if input.len() != Self::INPUT_LEN {
            return err!("Input length must be 213");
        }

        let gas = Self::gas_cost(input);
        if let Some(limit) = gas_limit {
            if limit < gas {
                return err!();
            }
        }

        let rounds = parse_round(input);
        let mut h = parse_h(input);
        let m = parse_m(input);
        let (t_0, t_1) = parse_t(input);
        let f = if input[212] == 1 {
            true
        } else if input[212] == 0 {
            false
        } else {
            return err!("Invalid f value");
        };

        compress(&mut h, m, [t_0, t_1], f, rounds as usize);

        let mut res = [0u8; 64];
        for (i, state) in h.iter().enumerate() {
            res[i * 8..(i + 1) * 8].copy_from_slice(&state.to_le_bytes());
        }

        Ok(PrecompileOutput {
            exit_status: ExitSucceed::Returned,
            cost:        gas,
            output:      res.to_vec(),
            logs:        vec![],
        })
    }

    fn gas_cost(input: &[u8]) -> u64 {
        let rounds = parse_round(input);
        (rounds as u64) * Self::GAS_PER_ROUND
    }
}

impl Blake2F {
    const GAS_PER_ROUND: u64 = 1;
    const INPUT_LEN: usize = 213;
}

fn parse_round(input: &[u8]) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&input[0..4]);
    u32::from_be_bytes(buf)
}

fn parse_h(input: &[u8]) -> [u64; 8] {
    let mut buf = [0u8; 64];
    buf.copy_from_slice(&input[4..68]);
    let mut h = [0u64; 8];

    for (i, state) in h.iter_mut().enumerate() {
        let mut temp = [0u8; 8];
        temp.copy_from_slice(&buf[(i * 8)..(i + 1) * 8]);
        *state = u64::from_le_bytes(temp);
    }

    h
}

fn parse_m(input: &[u8]) -> [u64; 16] {
    let mut buf = [0u8; 128];
    buf.copy_from_slice(&input[68..196]);
    let mut m = [0u64; 16];

    for (i, msg) in m.iter_mut().enumerate() {
        let mut temp = [0u8; 8];
        temp.copy_from_slice(&buf[(i * 8)..(i + 1) * 8]);
        *msg = u64::from_le_bytes(temp);
    }

    m
}

fn parse_t(input: &[u8]) -> (u64, u64) {
    let mut t_0_buf = [0u8; 8];
    t_0_buf.copy_from_slice(&input[196..204]);
    let t_0 = u64::from_le_bytes(t_0_buf);

    let mut t_1_buf = [0u8; 8];
    t_1_buf.copy_from_slice(&input[204..212]);
    let t_1 = u64::from_le_bytes(t_1_buf);

    (t_0, t_1)
}

const SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

#[inline(always)]
fn g(v: &mut [u64], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}

fn compress(h: &mut [u64; 8], m: [u64; 16], t: [u64; 2], f: bool, rounds: usize) {
    let mut v = [0u64; 16];
    v[..h.len()].copy_from_slice(h);
    v[h.len()..].copy_from_slice(&IV);

    v[12] ^= t[0];
    v[13] ^= t[1];

    if f {
        v[14] = !v[14]
    }

    for i in 0..rounds {
        let s = &SIGMA[i % 10];
        g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
        g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
        g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);

        g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
        g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
        g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
    }

    for i in 0..8 {
        h[i] ^= v[i] ^ v[i + 8];
    }
}
