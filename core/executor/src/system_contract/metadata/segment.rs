use protocol::ProtocolResult;

use crate::system_contract::{error::SystemScriptError, metadata::Epoch};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochSegment {
    seg: Vec<u64>,
}

impl EpochSegment {
    /// The genesis does not contain in any epoch.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        EpochSegment { seg: vec![0] }
    }

    /// Epoch segment requires the pushed endpoint is incremental.
    pub fn append_endpoint(&mut self, endpoint: u64) -> ProtocolResult<()> {
        if endpoint <= *self.seg.last().unwrap() {
            return Err(SystemScriptError::InvalidEpochEnd(endpoint).into());
        }

        self.seg.push(endpoint);
        Ok(())
    }

    /// Epoch segment records the epoch range as [seg[i], seg[i + 1]].
    pub fn get_epoch_number(&self, block_number: u64) -> ProtocolResult<Epoch> {
        for (e, s) in self.seg.windows(2).enumerate() {
            if s[0] < block_number && block_number <= s[1] {
                return Ok(e as Epoch);
            }
        }

        Err(SystemScriptError::FutureEpoch.into())
    }

    pub fn get_latest_epoch_number(&self) -> u64 {
        self.seg.len().saturating_sub(2) as u64
    }

    pub fn is_empty(&self) -> bool {
        self.seg.len() == 1
    }

    pub fn is_last_block_in_epoch(&self, block_number: u64) -> bool {
        self.seg.iter().any(|e| block_number == *e)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.seg.iter().flat_map(|s| s.to_be_bytes()).collect()
    }

    pub fn from_raw(raw: Vec<u8>) -> ProtocolResult<Self> {
        const U64_BYTES_LEN: usize = 8;

        if raw.is_empty() || raw[0] != 0 {
            return Err(SystemScriptError::DecodeEpochSegment(
                "First element in epoch segment missing or wrong".to_string(),
            )
            .into());
        }

        if raw.len() % U64_BYTES_LEN != 0 {
            return Err(SystemScriptError::DecodeEpochSegment(
                "Data length cannot divide 8".to_string(),
            )
            .into());
        }

        Ok(EpochSegment {
            seg: raw
                .chunks(U64_BYTES_LEN)
                .map(|r| {
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(r);
                    u64::from_be_bytes(buf)
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_segment() {
        let mut epochs = init_epoch_segment();

        test_get_epoch_number(&mut epochs);
        test_codec(&epochs);
    }

    fn init_epoch_segment() -> EpochSegment {
        let mut epochs = EpochSegment::new();
        epochs.append_endpoint(100).unwrap();
        epochs.append_endpoint(200).unwrap();
        epochs.append_endpoint(300).unwrap();
        epochs
    }

    fn test_get_epoch_number(epochs: &mut EpochSegment) {
        assert!(epochs.append_endpoint(150).is_err());
        assert!(epochs.get_epoch_number(0).is_err());
        assert_eq!(epochs.get_epoch_number(50).unwrap(), 0);
        assert_eq!(epochs.get_epoch_number(100).unwrap(), 0);
        assert_eq!(epochs.get_epoch_number(101).unwrap(), 1);
        assert_eq!(epochs.get_epoch_number(200).unwrap(), 1);
        assert_eq!(epochs.get_latest_epoch_number(), 2);
    }

    fn test_codec(epochs: &EpochSegment) {
        let raw = epochs.as_bytes();

        assert_eq!(EpochSegment::from_raw(raw).unwrap(), *epochs);

        assert!(EpochSegment::from_raw(vec![]).is_err());
        assert!(EpochSegment::from_raw([1u8; 8].into()).is_err());
        assert!(EpochSegment::from_raw([0u8; 8].into()).is_ok());
    }
}
