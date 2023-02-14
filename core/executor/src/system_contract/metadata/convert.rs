use protocol::types::{Hash, Hex, Metadata, MetadataVersion, ValidatorExtend};

use crate::system_contract::metadata::metadata_abi;

impl From<metadata_abi::Metadata> for Metadata {
    fn from(m: metadata_abi::Metadata) -> Metadata {
        Metadata {
            version:                    MetadataVersion {
                start: m.version.start,
                end:   m.version.end,
            },
            epoch:                      m.epoch,
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

impl From<Metadata> for metadata_abi::Metadata {
    fn from(m: Metadata) -> Self {
        metadata_abi::Metadata {
            version:                    m.version.into(),
            epoch:                      m.epoch,
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
            last_checkpoint_block_hash: m.last_checkpoint_block_hash.0,
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

impl From<MetadataVersion> for metadata_abi::MetadataVersion {
    fn from(version: MetadataVersion) -> Self {
        metadata_abi::MetadataVersion {
            start: version.start,
            end:   version.end,
        }
    }
}

impl From<ValidatorExtend> for metadata_abi::ValidatorExtend {
    fn from(ve: ValidatorExtend) -> Self {
        metadata_abi::ValidatorExtend {
            bls_pub_key:    ve.bls_pub_key.as_bytes().into(),
            pub_key:        ve.pub_key.as_bytes().into(),
            address:        ve.address,
            propose_weight: ve.propose_weight,
            vote_weight:    ve.vote_weight,
        }
    }
}
