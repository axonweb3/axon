mod adapter;
mod metadata_abi;

pub use crate::adapter::MetadataAdapterImpl;

use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;

use arc_swap::ArcSwap;
use ethers::abi::{self, Error as AbiError};
use ethers::core::abi::{AbiEncode, AbiType, InvalidOutputType, Tokenizable};
use parking_lot::RwLock;

use protocol::traits::{Context, MetadataControl, MetadataControlAdapter};
use protocol::types::{
    ExitReason, Hash, Header, Hex, Metadata, MetadataVersion, ValidatorExtend, H160,
};
use protocol::{Display, ProtocolError, ProtocolErrorKind, ProtocolResult};

type Epoch = u64;

lazy_static::lazy_static! {
    pub static ref EPOCH_LEN: ArcSwap<u64> = ArcSwap::new(Default::default());
}

pub struct MetadataController<Adapter> {
    adapter:          Arc<Adapter>,
    metadata_cache:   RwLock<BTreeMap<Epoch, Metadata>>,
    metadata_address: H160,
}

impl<Adapter> MetadataControl for MetadataController<Adapter>
where
    Adapter: MetadataControlAdapter + 'static,
{
    fn calc_epoch(&self, block_number: u64) -> u64 {
        calc_epoch(block_number)
    }

    fn need_change_metadata(&self, block_number: u64) -> bool {
        block_number % (**EPOCH_LEN.load()) == 0
    }

    fn update_metadata(&self, _ctx: Context, header: &Header) -> ProtocolResult<()> {
        let epoch = self.calc_epoch(header.number) + 1;
        let metadata = self.query_evm_metadata(epoch, header)?;
        let boundary = epoch.saturating_sub(20);

        let mut cache = self.metadata_cache.write();
        cache.retain(|&k, _| k < boundary);
        cache.insert(epoch, metadata);

        Ok(())
    }

    fn get_metadata(&self, _ctx: Context, header: &Header) -> ProtocolResult<Metadata> {
        let epoch = self.calc_epoch(header.number);
        let res = { self.metadata_cache.read().get(&epoch).cloned() };

        if res.is_none() {
            let metadata = self.query_evm_metadata(epoch, header)?;
            self.metadata_cache.write().insert(epoch, metadata.clone());
            return Ok(metadata);
        }

        Ok(res.unwrap())
    }

    fn get_metadata_unchecked(&self, _ctx: Context, block_number: u64) -> Metadata {
        self.metadata_cache
            .read()
            .get(&block_number)
            .cloned()
            .unwrap()
    }
}

impl<Adapter: MetadataControlAdapter> MetadataController<Adapter> {
    pub fn new(adapter: Arc<Adapter>, metadata_address: H160, epoch_len: u64) -> Self {
        EPOCH_LEN.swap(Arc::new(epoch_len));
        MetadataController {
            adapter,
            metadata_cache: RwLock::new(BTreeMap::new()),
            metadata_address,
        }
    }

    fn query_evm_metadata(&self, epoch: Epoch, header: &Header) -> ProtocolResult<Metadata> {
        let payload =
            metadata_abi::MetadataContractCalls::GetMetadata(metadata_abi::GetMetadataCall {
                epoch,
            });

        let res = self.adapter.call_evm(
            Context::new(),
            header,
            self.metadata_address,
            payload.encode(),
        )?;

        if !res.exit_reason.is_succeed() {
            return Err(MetadataError::CallEvm(res.exit_reason).into());
        }

        decode_resp_metadata(&res.ret)
    }
}

fn decode_resp_metadata(data: &[u8]) -> ProtocolResult<Metadata> {
    let tokens = abi::decode(&[metadata_abi::Metadata::param_type()], data)
        .map_err(MetadataError::AbiDecode)?;
    let res = metadata_abi::Metadata::from_token(tokens[0].clone())
        .map_err(MetadataError::InvalidTokenType)?;
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

fn calc_epoch(block_number: u64) -> u64 {
    block_number / (**EPOCH_LEN.load())
}

#[derive(Debug, Display)]
pub enum MetadataError {
    #[display(fmt = "Abi decode error {:?}", _0)]
    AbiDecode(AbiError),

    #[display(fmt = "Invalid token type {:?}", _0)]
    InvalidTokenType(InvalidOutputType),

    #[display(fmt = "Call EVM exit {:?}", _0)]
    CallEvm(ExitReason),
}

impl Error for MetadataError {}

impl From<MetadataError> for ProtocolError {
    fn from(error: MetadataError) -> ProtocolError {
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
