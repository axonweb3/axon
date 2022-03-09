mod adapter_tests;
mod controller_tests;

use std::sync::Arc;

use protocol::types::{Metadata, MetadataVersion, ValidatorExtend};

use crate::metadata_abi as abi;
use crate::{calc_epoch, EPOCH_LEN};

impl From<MetadataVersion> for abi::MetadataVersion {
    fn from(version: MetadataVersion) -> Self {
        abi::MetadataVersion {
            start: version.start,
            end:   version.end,
        }
    }
}

impl From<ValidatorExtend> for abi::ValidatorExtend {
    fn from(ve: ValidatorExtend) -> Self {
        abi::ValidatorExtend {
            bls_pub_key:    ve.bls_pub_key.as_bytes().into(),
            pub_key:        ve.pub_key.as_bytes().into(),
            address:        ve.address,
            propose_weight: ve.propose_weight,
            vote_weight:    ve.vote_weight,
        }
    }
}

impl From<Metadata> for abi::Metadata {
    fn from(m: Metadata) -> Self {
        abi::Metadata {
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
//     Abigen::new("MetadataContract", "./metadata.abi")
//         .unwrap()
//         .generate()
//         .unwrap()
//         .write_to_file("./src/metadata_abi.rs")
//         .unwrap();
// }
