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
/// So that the default value of `AddressSource` means using
/// `blake2b_256(input[0].lock().as_bytes())` as `keccak()` input.
#[derive(
    RlpEncodable, RlpDecodable, Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq, Eq,
)]
pub struct AddressSource {
    pub type_: u8,
    pub index: u32,
}

#[derive(Clone, Debug)]
pub enum SignatureR {
    ByRef(CKBTxMockByRef),
    ByRefAndOneInput(CKBTxMockByRefAndOneInput),
}

impl SignatureR {
    pub fn new_by_ref(
        cell_deps: Vec<CellDep>,
        header_deps: Vec<H256>,
        out_points: Vec<OutPoint>,
        out_point_addr_source: AddressSource,
    ) -> Self {
        SignatureR::ByRef(CKBTxMockByRef {
            cell_deps,
            header_deps,
            out_points,
            out_point_addr_source,
        })
    }

    pub fn decode(data: &[u8]) -> ProtocolResult<Self> {
        if data.is_empty() {
            return Err(TypesError::SignatureRIsEmpty.into());
        }

        let ret = match data[0] {
            1u8 => SignatureR::ByRef(CKBTxMockByRef::decode(&data[1..])?),
            2u8 => SignatureR::ByRefAndOneInput(CKBTxMockByRefAndOneInput::decode(&data[1..])?),
            _ => return Err(TypesError::InvalidSignatureRType.into()),
        };

        if ret.address_source().type_ > 1 {
            return Err(TypesError::InvalidAddressSourceType.into());
        }

        Ok(ret)
    }

    pub fn inputs_len(&self) -> usize {
        match self {
            SignatureR::ByRef(i) => i.out_points.len(),
            SignatureR::ByRefAndOneInput(_) => 1usize,
        }
    }

    pub fn address_source(&self) -> AddressSource {
        match self {
            SignatureR::ByRef(i) => i.out_point_addr_source,
            SignatureR::ByRefAndOneInput(i) => i.out_point_addr_source,
        }
    }

    pub fn cell_deps(&self) -> &[CellDep] {
        match self {
            SignatureR::ByRef(i) => &i.cell_deps,
            SignatureR::ByRefAndOneInput(i) => &i.cell_deps,
        }
    }

    pub fn header_deps(&self) -> &[H256] {
        match self {
            SignatureR::ByRef(i) => &i.header_deps,
            SignatureR::ByRefAndOneInput(i) => &i.header_deps,
        }
    }

    pub(crate) fn out_points(&self) -> &[OutPoint] {
        match self {
            SignatureR::ByRef(i) => &i.out_points,
            _ => unreachable!(),
        }
    }

    pub fn dummy_input(&self) -> Option<CellWithData> {
        match self {
            SignatureR::ByRefAndOneInput(i) => Some(i.input_lock.clone()),
            SignatureR::ByRef(_) => None,
        }
    }

    pub fn is_only_by_ref(&self) -> bool {
        match self {
            SignatureR::ByRef(_) => true,
            SignatureR::ByRefAndOneInput(_) => false,
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CKBTxMockByRef {
    pub cell_deps:             Vec<CellDep>,
    pub header_deps:           Vec<H256>,
    pub out_points:            Vec<OutPoint>,
    pub out_point_addr_source: AddressSource,
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CKBTxMockByRefAndOneInput {
    pub cell_deps:             Vec<CellDep>,
    pub header_deps:           Vec<H256>,
    pub input_lock:            CellWithData,
    pub out_point_addr_source: AddressSource,
}

#[derive(RlpEncodable, RlpDecodable, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CellWithData {
    pub type_script: Option<Script>,
    pub lock_script: Script,
    pub data:        Bytes,
}

impl CellWithData {
    pub fn capacity(&self) -> u64 {
        let capacity = self
            .type_script
            .as_ref()
            .map(|s| s.len())
            .unwrap_or_default()
            + self.lock_script.len()
            + self.data.len();
        capacity as u64
    }

    pub fn lock_script(&self) -> packed::Script {
        (&self.lock_script).into()
    }

    pub fn type_script(&self) -> packed::ScriptOpt {
        self.type_script.as_ref().map(packed::Script::from).pack()
    }

    pub fn lock_script_hash(&self) -> [u8; 32] {
        ckb_hash::blake2b_256(self.lock_script().as_slice())
    }

    pub fn type_script_hash(&self) -> Option<[u8; 32]> {
        self.type_script()
            .to_opt()
            .map(|s| ckb_hash::blake2b_256(s.as_slice()))
    }
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
