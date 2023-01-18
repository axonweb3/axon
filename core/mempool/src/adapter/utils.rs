use rlp_derive::{RlpDecodable, RlpEncodable};

use protocol::types::{Bytes, H256};

use core_executor::precompiles::CellDep;

#[derive(RlpEncodable, RlpDecodable, Clone, Debug)]
pub struct R {
    pub out_points:  Vec<OutPoint>,
    pub cell_deps:   Vec<CellDep>,
    pub header_deps: Vec<H256>,
}

#[derive(RlpEncodable, RlpDecodable, Clone, Debug)]
pub struct S {
    pub witnesses: Vec<SimplifiedWitness>,
}

#[derive(RlpEncodable, RlpDecodable, Clone, Debug)]
pub struct OutPoint {
    pub tx_hash: H256,
    pub index:   u32,
}

#[derive(RlpEncodable, RlpDecodable, Clone, Debug)]
pub struct SimplifiedWitness {
    pub lock:       Option<Bytes>,
    pub input_type: Option<Bytes>,
}

#[derive(RlpEncodable, RlpDecodable, Clone, Debug)]
pub struct CellDepWithPubKey {
    pub cell_dep: CellDep,
    pub pub_key:  Bytes,
}
