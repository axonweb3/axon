use ckb_types::{packed, prelude::*};
use derive_more::Display;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::types::{Bytes, H256};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VMResp {
    pub exit_code: i8,
    pub cycles:    u64,
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CellWithData {
    pub type_script: Option<Script>,
    pub lock_script: Script,
    pub data:        Bytes,
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Script {
    pub code_hash: H256,
    pub args:      Bytes,
    pub hash_type: u8,
}

impl From<&Script> for packed::Script {
    fn from(s: &Script) -> Self {
        packed::ScriptBuilder::default()
            .code_hash(s.code_hash.0.pack())
            .args(s.args.pack())
            .hash_type(s.hash_type.into())
            .build()
    }
}

impl Script {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.args.len() + 32 + 1
    }
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SignatureS {
    pub witnesses: Vec<Witness>,
}

impl SignatureS {
    pub fn new(witnesses: Vec<Witness>) -> Self {
        SignatureS { witnesses }
    }
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CellDepWithPubKey {
    pub cell_dep: CellDep,
    pub pub_key:  Bytes,
}

#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Display,
)]
#[display(fmt = "OutPoint {{ tx_hash: {:#x}, index: {} }}", tx_hash, index)]
pub struct OutPoint {
    pub tx_hash: H256,
    pub index:   u32,
}

impl From<&packed::OutPoint> for OutPoint {
    fn from(out_point: &packed::OutPoint) -> Self {
        OutPoint {
            tx_hash: H256(out_point.tx_hash().unpack().0),
            index:   out_point.index().unpack(),
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Witness {
    pub lock:        Option<Bytes>,
    pub input_type:  Option<Bytes>,
    pub output_type: Option<Bytes>,
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CellDep {
    pub tx_hash:  H256,
    pub index:    u32,
    pub dep_type: u8,
}

impl From<&CellDep> for packed::CellDep {
    fn from(dep: &CellDep) -> packed::CellDep {
        packed::CellDepBuilder::default()
            .out_point(
                packed::OutPointBuilder::default()
                    .tx_hash(dep.tx_hash.0.pack())
                    .index(dep.index.pack())
                    .build(),
            )
            .dep_type(packed::Byte::new(dep.dep_type))
            .build()
    }
}
