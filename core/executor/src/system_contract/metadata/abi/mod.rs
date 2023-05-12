pub mod metadata_abi;

use protocol::types::{
    CkbRelatedInfo, Hex, Metadata, MetadataVersion, ProposeCount, ValidatorExtend, H256,
};

impl From<metadata_abi::Metadata> for Metadata {
    fn from(value: metadata_abi::Metadata) -> Metadata {
        Metadata {
            version:         MetadataVersion {
                start: value.version.start,
                end:   value.version.end,
            },
            epoch:           value.epoch,
            gas_limit:       value.gas_limit,
            gas_price:       value.gas_price,
            interval:        value.interval,
            verifier_list:   value.verifier_list.into_iter().map(Into::into).collect(),
            propose_ratio:   value.propose_ratio,
            prevote_ratio:   value.prevote_ratio,
            precommit_ratio: value.precommit_ratio,
            brake_ratio:     value.brake_ratio,
            tx_num_limit:    value.tx_num_limit,
            max_tx_size:     value.max_tx_size,
            propose_counter: value.propose_counter.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Metadata> for metadata_abi::Metadata {
    fn from(value: Metadata) -> Self {
        metadata_abi::Metadata {
            version:         value.version.into(),
            epoch:           value.epoch,
            gas_limit:       value.gas_limit,
            gas_price:       value.gas_price,
            interval:        value.interval,
            verifier_list:   value.verifier_list.into_iter().map(Into::into).collect(),
            propose_ratio:   value.propose_ratio,
            prevote_ratio:   value.prevote_ratio,
            precommit_ratio: value.precommit_ratio,
            brake_ratio:     value.brake_ratio,
            tx_num_limit:    value.tx_num_limit,
            max_tx_size:     value.max_tx_size,
            propose_counter: value.propose_counter.into_iter().map(Into::into).collect(),
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

impl From<metadata_abi::ValidatorExtend> for ValidatorExtend {
    fn from(value: metadata_abi::ValidatorExtend) -> ValidatorExtend {
        ValidatorExtend {
            bls_pub_key:    Hex::encode(value.bls_pub_key),
            pub_key:        Hex::encode(value.pub_key),
            address:        value.address,
            propose_weight: value.propose_weight,
            vote_weight:    value.vote_weight,
        }
    }
}

impl From<ValidatorExtend> for metadata_abi::ValidatorExtend {
    fn from(value: ValidatorExtend) -> Self {
        metadata_abi::ValidatorExtend {
            bls_pub_key:    value.bls_pub_key.as_bytes().into(),
            pub_key:        value.pub_key.as_bytes().into(),
            address:        value.address,
            propose_weight: value.propose_weight,
            vote_weight:    value.vote_weight,
        }
    }
}

impl From<ProposeCount> for metadata_abi::ProposeCount {
    fn from(pc: ProposeCount) -> Self {
        metadata_abi::ProposeCount {
            address: pc.address,
            count:   pc.count,
        }
    }
}

impl From<metadata_abi::ProposeCount> for ProposeCount {
    fn from(value: metadata_abi::ProposeCount) -> Self {
        ProposeCount {
            address: value.address,
            count:   value.count,
        }
    }
}

impl From<CkbRelatedInfo> for metadata_abi::CkbRelatedInfo {
    fn from(value: CkbRelatedInfo) -> Self {
        metadata_abi::CkbRelatedInfo {
            metadata_type_id:       value.metadata_type_id.0,
            checkpoint_type_id:     value.checkpoint_type_id.0,
            stake_token_type_id:    value.stake_token_type_id.0,
            delegate_token_type_id: value.delegate_token_type_id.0,
        }
    }
}

impl From<metadata_abi::CkbRelatedInfo> for CkbRelatedInfo {
    fn from(value: metadata_abi::CkbRelatedInfo) -> Self {
        CkbRelatedInfo {
            metadata_type_id:       H256(value.metadata_type_id),
            checkpoint_type_id:     H256(value.checkpoint_type_id),
            stake_token_type_id:    H256(value.stake_token_type_id),
            delegate_token_type_id: H256(value.delegate_token_type_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::core::abi::AbiEncode;

    #[test]
    fn test_print_ckb_related_info() {
        let info: metadata_abi::CkbRelatedInfo = CkbRelatedInfo {
            metadata_type_id:       H256::from([1u8; 32]),
            checkpoint_type_id:     H256::from([2u8; 32]),
            stake_token_type_id:    H256::from([3u8; 32]),
            delegate_token_type_id: H256::from([4u8; 32]),
        }
        .into();

        let raw = AbiEncode::encode(info);
        println!("{:?}", raw);
    }
}
