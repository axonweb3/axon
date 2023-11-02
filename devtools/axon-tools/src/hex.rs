use crate::Error;

pub fn hex_encode<T: AsRef<[u8]>>(src: T) -> String {
    faster_hex::hex_string(src.as_ref())
}

pub fn hex_decode(src: &str) -> Result<Vec<u8>, Error> {
    if src.is_empty() {
        return Ok(Vec::new());
    }

    let src = if src.starts_with("0x") {
        src.split_at(2).1
    } else {
        src
    };

    let src = src.as_bytes();
    let mut ret = vec![0u8; src.len() / 2];
    faster_hex::hex_decode(src, &mut ret)?;

    Ok(ret)
}
