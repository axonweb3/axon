use serde::{Deserialize, Serialize};

use crate::types::{Bloom, Bytes, Hash, MerkleRoot, SignedTransaction, H160, H64, U256};

pub type BlockNumber = u64;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub header:    Header,
    pub tx_hashes: Vec<Hash>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub prev_hash:                  Hash,
    pub proposer:                   H160,
    pub state_root:                 MerkleRoot,
    pub transactions_root:          MerkleRoot,
    pub signed_txs_hash:            Hash,
    pub receipts_root:              MerkleRoot,
    pub log_bloom:                  Bloom,
    pub difficulty:                 U256,
    pub timestamp:                  u64,
    pub number:                     BlockNumber,
    pub gas_used:                   U256,
    pub gas_limit:                  U256,
    pub extra_data:                 Bytes,
    pub mixed_hash:                 Option<Hash>,
    pub nonce:                      H64,
    pub base_fee_per_gas:           U256,
    pub proof:                      Proof,
    pub last_checkpoint_block_hash: Hash,
    pub chain_id:                   u64,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub number:     u64,
    pub round:      u64,
    pub block_hash: Hash,
    pub signature:  Bytes,
    pub bitmap:     Bytes,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pill {
    pub block:          Block,
    pub propose_hashes: Vec<Hash>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Genesis {
    pub block:    Block,
    pub rich_txs: Vec<SignedTransaction>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RichBlock {
    pub block: Block,
    pub txs:   Vec<SignedTransaction>,
}

#[cfg(test)]
mod tests {
    use crate::types::{
        Block, Genesis, Header, Hex, Metadata, MetadataVersion, ValidatorExtend, H160,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn time_now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    #[test]
    fn print_genesis() {
        let genesis = Genesis {
            rich_txs: vec![],
            block:    Block {
                tx_hashes: vec![],
                header:    Header {
                    prev_hash:                  Default::default(),
                    proposer:                   Default::default(),
                    state_root:                 Default::default(),
                    transactions_root:          Default::default(),
                    signed_txs_hash:            Default::default(),
                    receipts_root:              Default::default(),
                    log_bloom:                  Default::default(),
                    difficulty:                 Default::default(),
                    timestamp:                  time_now(),
                    number:                     0,
                    gas_used:                   Default::default(),
                    gas_limit:                  Default::default(),
                    extra_data:                 Default::default(),
                    mixed_hash:                 Default::default(),
                    nonce:                      Default::default(),
                    base_fee_per_gas:           Default::default(),
                    proof:                      Default::default(),
                    last_checkpoint_block_hash: Default::default(),
                    chain_id:                   0,
                },
            },
        };

        println!("{}", serde_json::to_string(&genesis).unwrap());
    }

    #[test]
    fn print_metadata() {
        let metadata = Metadata {
            chain_id: 0u64.into(),
            version: MetadataVersion::new(0, 1000000000),
            timeout_gap: 1000,
            common_ref: Hex::from_string("0x6c747758636859487038".to_string()).unwrap(),
            gas_limit: 4294967295,
            gas_price: 1,
            interval: 3000,
            propose_ratio: 15,
            prevote_ratio: 10,
            precommit_ratio: 10,
            brake_ratio: 10,
            tx_num_limit: 20000,
            max_tx_size: 1024,
            verifier_list: vec![ValidatorExtend {
                bls_pub_key: Hex::from_string("0x04102947214862a503c73904deb5818298a186d68c7907bb609583192a7de6331493835e5b8281f4d9ee705537c0e765580e06f86ddce5867812fceb42eecefd209f0eddd0389d6b7b0100f00fb119ef9ab23826c6ea09aadcc76fa6cea6a32724".to_string()).unwrap(),
                pub_key: Hex::from_string("0x02ef0cb0d7bc6c18b4bea1f5908d9106522b35ab3c399369605d4242525bda7e60".to_string()).unwrap(),
                address: H160::default(),
                propose_weight: 1,
                vote_weight: 1,
            }],
            last_checkpoint_block_hash : Default::default(),
        };

        println!("{}", serde_json::to_string(&metadata).unwrap());
    }
}
