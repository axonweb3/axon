use bytes::Bytes;
use ckb_types::{packed, prelude::*};
use ethereum_types::H256;
use rlp_derive::{RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

use crate::{codec::ProtocolCodec, types::TypesError, ProtocolResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct VMResp {
    pub exit_code: i8,
    pub cycles:    u64,
}

/// The address mapping for calculate an Axon address by the input cell, which
/// is `keccak(input[index].content).into()`. The `type_` field means the type
/// of content to calculate hash with the following rules:
/// `0u8`: use lock script hash.
/// `1u8`: use type script hash.
/// So that the default value of `AddressMapping` means using
/// `blake2b_256(input[0].lock().as_bytes())` as `keccak()` input.
#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq,
)]
pub struct AddressMapping {
    pub type_: u8,
    pub index: u32,
}

#[derive(Clone, Debug)]
pub enum SignatureR {
    RealityInput(RealityInput),
    DummyInput(DummyInput),
}

impl SignatureR {
    pub fn new_reality(
        cell_deps: Vec<CellDep>,
        header_deps: Vec<H256>,
        out_points: Vec<OutPoint>,
        address_map: AddressMapping,
    ) -> Self {
        SignatureR::RealityInput(RealityInput {
            cell_deps,
            header_deps,
            out_points,
            address_map,
        })
    }

    pub fn decode(data: &[u8]) -> ProtocolResult<Self> {
        if data.is_empty() {
            return Err(TypesError::SignatureRIsEmpty.into());
        }

        match data[0] {
            1u8 => Ok(SignatureR::RealityInput(RealityInput::decode(&data[1..])?)),
            2u8 => Ok(SignatureR::DummyInput(DummyInput::decode(&data[1..])?)),
            _ => Err(TypesError::InvalidSignatureRType.into()),
        }
    }

    pub fn input_len(&self) -> usize {
        match self {
            SignatureR::RealityInput(i) => i.out_points.len(),
            SignatureR::DummyInput(_) => 0usize,
        }
    }

    pub fn address_mapping(&self) -> AddressMapping {
        match self {
            SignatureR::RealityInput(i) => i.address_map,
            SignatureR::DummyInput(_) => AddressMapping::default(),
        }
    }

    pub fn cell_deps(&self) -> &[CellDep] {
        match self {
            SignatureR::RealityInput(i) => &i.cell_deps,
            SignatureR::DummyInput(i) => &i.cell_deps,
        }
    }

    pub fn header_deps(&self) -> &[H256] {
        match self {
            SignatureR::RealityInput(i) => &i.header_deps,
            SignatureR::DummyInput(i) => &i.header_deps,
        }
    }

    pub(crate) fn reality_inputs(&self) -> &[OutPoint] {
        match self {
            SignatureR::RealityInput(i) => &i.out_points,
            _ => unreachable!(),
        }
    }

    pub fn dummy_input(&self) -> Option<InputLock> {
        match self {
            SignatureR::DummyInput(i) => Some(i.input_lock.clone()),
            SignatureR::RealityInput(_) => None,
        }
    }

    pub fn is_reality(&self) -> bool {
        match self {
            SignatureR::RealityInput(_) => true,
            SignatureR::DummyInput(_) => false,
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RealityInput {
    pub cell_deps:   Vec<CellDep>,
    pub header_deps: Vec<H256>,
    pub out_points:  Vec<OutPoint>,
    pub address_map: AddressMapping,
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DummyInput {
    pub cell_deps:   Vec<CellDep>,
    pub header_deps: Vec<H256>,
    pub input_lock:  InputLock,
    // pub address_map: AddressMapping,
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct InputLock {
    pub lock_code_hash: H256,
    pub lock_args:      Bytes,
    pub hash_type:      u8,
    pub data:           Bytes,
}

impl InputLock {
    pub fn capacity(&self) -> u64 {
        let capacity = 32 + self.lock_args.len() + 1 + self.data.len();
        capacity as u64
    }

    pub fn as_script(&self) -> packed::Script {
        packed::ScriptBuilder::default()
            .code_hash(self.lock_code_hash.0.pack())
            .args(self.lock_args.pack())
            .hash_type(self.hash_type.into())
            .build()
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

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
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
    pub input_type:  Option<Bytes>,
    pub output_type: Option<Bytes>,
    pub lock:        Option<Bytes>,
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
