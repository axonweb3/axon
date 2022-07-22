use protocol::types::{Bloom, Hasher, Log, U256};

const FUNC_SELECTOR_LEN: usize = 4;
const U256_BE_BYTES_LEN: usize = 32;
const REVERT_MSG_LEN_OFFSET: usize = FUNC_SELECTOR_LEN + U256_BE_BYTES_LEN;
const REVERT_EFFECT_MSG_OFFSET: usize = REVERT_MSG_LEN_OFFSET + U256_BE_BYTES_LEN;
const BLOOM_BYTE_LENGTH: usize = 256;

pub fn decode_revert_msg(input: &[u8]) -> String {
    if input.len() < REVERT_EFFECT_MSG_OFFSET {
        return String::new();
    }

    let end_offset = REVERT_EFFECT_MSG_OFFSET
        + U256::from_big_endian(&input[REVERT_MSG_LEN_OFFSET..REVERT_EFFECT_MSG_OFFSET]).as_usize();

    if input.len() < end_offset {
        return String::new();
    }

    let reason = String::from_iter(
        input[REVERT_EFFECT_MSG_OFFSET..end_offset]
            .iter()
            .map(|i| *i as char),
    );

    format!("execution reverted: {}", reason)
}

pub fn logs_bloom<'a, I>(logs: I) -> Bloom
where
    I: Iterator<Item = &'a Log>,
{
    let mut bloom = Bloom::zero();
    for log in logs {
        m3_2048(&mut bloom, log.address.as_bytes());
        for topic in log.topics.iter() {
            m3_2048(&mut bloom, topic.as_bytes());
        }
    }
    bloom
}

fn m3_2048(bloom: &mut Bloom, x: &[u8]) {
    let hash = Hasher::digest(x).0;
    for i in [0, 2, 4] {
        let bit = (hash[i + 1] as usize + ((hash[i] as usize) << 8)) & 0x7FF;
        bloom.0[BLOOM_BYTE_LENGTH - 1 - bit / 8] |= 1 << (bit % 8);
    }
}
