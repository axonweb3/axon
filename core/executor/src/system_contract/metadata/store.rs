use std::sync::Arc;

use protocol::types::{Metadata, H256};
use protocol::{codec::ProtocolCodec, ProtocolResult};

use crate::system_contract::metadata::{
    segment::EpochSegment, CURRENT_METADATA_ROOT, EPOCH_SEGMENT_KEY, METADATA_DB,
};
use crate::system_contract::{error::SystemScriptError, image_cell::RocksTrieDB};
use crate::MPTTrie;

pub struct MetadataStore {
    pub trie: MPTTrie<RocksTrieDB>,
}

impl MetadataStore {
    pub fn new() -> ProtocolResult<Self> {
        let trie_db = match METADATA_DB.get() {
            Some(db) => db,
            None => return Err(SystemScriptError::TrieDbNotInit.into()),
        };

        let root = **CURRENT_METADATA_ROOT.load();

        let trie = if root == H256::default() {
            let mut m = MPTTrie::new(Arc::clone(trie_db));
            m.insert(
                EPOCH_SEGMENT_KEY.as_bytes(),
                &EpochSegment::new().as_bytes(),
            )?;
            m
        } else {
            match MPTTrie::from_root(root, Arc::clone(trie_db)) {
                Ok(m) => m,
                Err(e) => return Err(SystemScriptError::RestoreMpt(e.to_string()).into()),
            }
        };

        Ok(MetadataStore { trie })
    }

    pub fn append_metadata(&mut self, metadata: Metadata) -> ProtocolResult<()> {
        let mut epoch_segment = EpochSegment::from_raw(
            self.trie
                .get(EPOCH_SEGMENT_KEY.as_bytes())?
                .unwrap()
                .to_vec(),
        )?;
        epoch_segment.push_endpoint(metadata.version.end)?;

        self.trie
            .insert(EPOCH_SEGMENT_KEY.as_bytes(), &epoch_segment.as_bytes())?;
        self.trie
            .insert(&metadata.epoch.to_be_bytes(), &metadata.encode()?)?;
        let new_root = self.trie.commit()?;
        CURRENT_METADATA_ROOT.swap(Arc::new(new_root));

        Ok(())
    }

    pub fn get_epoch_segment(&self) -> ProtocolResult<EpochSegment> {
        let raw = self.trie.get(EPOCH_SEGMENT_KEY.as_bytes())?.unwrap();
        EpochSegment::from_raw(raw.to_vec())
    }

    pub fn get_metadata(&self, epoch: u64) -> ProtocolResult<Metadata> {
        let raw = self
            .trie
            .get(&epoch.to_be_bytes())?
            .ok_or_else(|| SystemScriptError::FutureEpoch)?;
        Metadata::decode(raw)
    }
}
