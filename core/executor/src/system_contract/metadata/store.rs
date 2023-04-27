use std::collections::BTreeMap;
use std::sync::Arc;

use protocol::types::{CkbRelatedInfo, Metadata, H160, H256};
use protocol::{codec::ProtocolCodec, ProtocolResult};

use crate::system_contract::error::SystemScriptError;
use crate::system_contract::metadata::CKB_RELATED_INFO_KEY;
use crate::system_contract::metadata::{
    segment::EpochSegment, CURRENT_METADATA_ROOT, EPOCH_SEGMENT_KEY,
};
use crate::system_contract::trie_db::RocksTrieDB;
use crate::system_contract::METADATA_DB;
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

    pub fn set_ckb_related_info(&mut self, info: &CkbRelatedInfo) -> ProtocolResult<()> {
        self.trie
            .insert(CKB_RELATED_INFO_KEY.as_bytes(), &info.encode()?)
    }

    pub fn append_metadata(&mut self, metadata: &Metadata) -> ProtocolResult<()> {
        let mut epoch_segment = EpochSegment::from_raw(
            self.trie
                .get(EPOCH_SEGMENT_KEY.as_bytes())?
                .unwrap()
                .to_vec(),
        )?;

        // epoch should be 0 at the first time
        // and should be the latest epoch + 1 after that.
        let latest_epoch_number = epoch_segment.get_latest_epoch_number();
        if (epoch_segment.is_empty() && metadata.epoch != 0)
            || (!epoch_segment.is_empty() && metadata.epoch != latest_epoch_number + 1)
        {
            return Err(SystemScriptError::PastEpoch.into());
        }

        let map = metadata
            .verifier_list
            .iter()
            .map(|v| (v.address, 0u64))
            .collect::<BTreeMap<_, _>>();
        let mut metadata = metadata.clone();
        metadata.propose_counter = map.into_iter().map(Into::into).collect();

        epoch_segment.append_endpoint(metadata.version.end)?;

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

    pub fn get_metadata_by_block_number(&self, block_number: u64) -> ProtocolResult<Metadata> {
        let epoch = self.get_epoch_by_block_number(block_number)?;
        self.get_metadata(epoch)
    }

    pub fn get_ckb_related_info(&self) -> ProtocolResult<CkbRelatedInfo> {
        let raw = self
            .trie
            .get(CKB_RELATED_INFO_KEY.as_bytes())?
            .expect("ckb related info should exist");
        CkbRelatedInfo::decode(raw)
    }

    pub fn update_propose_count(
        &mut self,
        block_number: u64,
        proposer: &H160,
    ) -> ProtocolResult<()> {
        let mut metadata = self.get_metadata_by_block_number(block_number)?;
        let mut map = metadata
            .propose_counter
            .iter()
            .map(|c| (c.address, c.count))
            .collect::<BTreeMap<_, _>>();
        *map.get_mut(proposer).unwrap() += 1;
        metadata.propose_counter = map.into_iter().map(Into::into).collect();

        self.trie
            .insert(&metadata.epoch.to_be_bytes(), &metadata.encode()?)?;
        let new_root = self.trie.commit()?;
        CURRENT_METADATA_ROOT.swap(Arc::new(new_root));

        Ok(())
    }

    fn get_epoch_by_block_number(&self, block_number: u64) -> ProtocolResult<u64> {
        self.get_epoch_segment()?.get_epoch_number(block_number)
    }
}
