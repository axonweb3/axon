pub mod metadata_abi;

pub use ethers::core::abi::{AbiDecode, AbiEncode, AbiType, InvalidOutputType, Tokenizable};

use std::collections::BTreeMap;
use std::error::Error;

use arc_swap::ArcSwap;
use ethers::abi::{self, Error as AbiError};
use parking_lot::RwLock;

use protocol::types::{Hash, Hex, Metadata, MetadataVersion, ValidatorExtend};
use protocol::{Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

lazy_static::lazy_static! {
    pub static ref EPOCH_LEN: ArcSwap<u64> = ArcSwap::new(Default::default());
    pub static ref METADATA: RwLock<BTreeMap<u64, Metadata>> = RwLock::new(Default::default());
}

pub fn calc_epoch(block_number: u64) -> u64 {
    block_number / (**EPOCH_LEN.load())
}

pub fn need_change_metadata(block_number: u64) -> bool {
    block_number % (**EPOCH_LEN.load()) == 0
}

pub fn upload_and_clean_outdated(block_number: u64, metadata: Metadata) {
    let boundary = calc_epoch(block_number).saturating_sub(1);
    let mut m = METADATA.write();
    m.retain(|&k, _| k < boundary);
    m.insert(calc_epoch(metadata.version.start), metadata);
}

pub fn get_metadata(block_number: u64) -> Option<Metadata> {
    let epoch = calc_epoch(block_number);
    METADATA.read().get(&epoch).cloned()
}

pub fn decode_resp_metadata(data: &[u8]) -> ProtocolResult<Metadata> {
    let tokens = abi::decode(&[metadata_abi::Metadata::param_type()], data)
        .map_err(ContractError::AbiDecode)?;
    let res = metadata_abi::Metadata::from_token(tokens[0].clone())
        .map_err(ContractError::InvalidTokenType)?;
    Ok(res.into())
}

impl From<metadata_abi::Metadata> for Metadata {
    fn from(m: metadata_abi::Metadata) -> Metadata {
        Metadata {
            version:                    MetadataVersion {
                start: m.version.start,
                end:   m.version.end,
            },
            gas_limit:                  m.gas_limit,
            gas_price:                  m.gas_price,
            interval:                   m.interval,
            verifier_list:              m.verifier_list.into_iter().map(Into::into).collect(),
            propose_ratio:              m.propose_ratio,
            prevote_ratio:              m.prevote_ratio,
            precommit_ratio:            m.precommit_ratio,
            brake_ratio:                m.brake_ratio,
            tx_num_limit:               m.tx_num_limit,
            max_tx_size:                m.max_tx_size,
            last_checkpoint_block_hash: Hash::from_slice(&m.last_checkpoint_block_hash),
        }
    }
}

impl From<metadata_abi::ValidatorExtend> for ValidatorExtend {
    fn from(v: metadata_abi::ValidatorExtend) -> ValidatorExtend {
        ValidatorExtend {
            bls_pub_key:    Hex::encode(v.bls_pub_key),
            pub_key:        Hex::encode(v.pub_key),
            address:        v.address,
            propose_weight: v.propose_weight,
            vote_weight:    v.vote_weight,
        }
    }
}

#[derive(Debug, Display)]
pub enum ContractError {
    #[display(fmt = "Abi decode error {:?}", _0)]
    AbiDecode(AbiError),

    #[display(fmt = "Invalid token type {:?}", _0)]
    InvalidTokenType(InvalidOutputType),
}

impl Error for ContractError {}

impl From<ContractError> for ProtocolError {
    fn from(error: ContractError) -> ProtocolError {
        ProtocolError::new(ProtocolErrorKind::Contract, Box::new(error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_calc_epoch() {
        EPOCH_LEN.swap(Arc::new(100u64));

        assert_eq!(calc_epoch(1), 0);
        assert_eq!(calc_epoch(99), 0);
        assert_eq!(calc_epoch(100), 1);
        assert_eq!(calc_epoch(101), 1);
        assert_eq!(calc_epoch(200), 2);
    }

    // #[test]
    // fn test_abi() {
    //     ethers_contract_abigen::Abigen::new("MetadataContract",
    // "./metadata.abi")         .unwrap()
    //         .generate()
    //         .unwrap()
    //         .write_to_file("./src/metadata_abi.rs")
    //         .unwrap();
    // }
}
