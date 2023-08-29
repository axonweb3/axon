use std::collections::BTreeMap;
use std::sync::Arc;

use protocol::trie::Trie as _;
use protocol::types::{CkbRelatedInfo, Metadata, H160, H256};
use protocol::{codec::ProtocolCodec, ProtocolResult};

use crate::system_contract::metadata::{
    segment::EpochSegment, CKB_RELATED_INFO_KEY, EPOCH_SEGMENT_KEY,
};
use crate::system_contract::{error::SystemScriptError, METADATA_DB};
use crate::{adapter::RocksTrieDB, MPTTrie, CURRENT_METADATA_ROOT};

use super::metadata_abi::ConsensusConfig;

pub struct MetadataStore {
    pub trie: MPTTrie<RocksTrieDB>,
}

impl MetadataStore {
    pub fn new(root: H256) -> ProtocolResult<Self> {
        let trie_db = {
            let lock = METADATA_DB.read().clone();
            match lock {
                Some(db) => db,
                None => return Err(SystemScriptError::TrieDbNotInit.into()),
            }
        };

        let trie = if root == H256::default() {
            let mut m = MPTTrie::new(Arc::clone(&trie_db));
            m.insert(
                EPOCH_SEGMENT_KEY.as_bytes().to_vec(),
                EpochSegment::new().as_bytes(),
            )?;
            m
        } else {
            match MPTTrie::from_root(root, Arc::clone(&trie_db)) {
                Ok(m) => m,
                Err(e) => return Err(SystemScriptError::RestoreMpt(e.to_string()).into()),
            }
        };

        Ok(MetadataStore { trie })
    }

    pub fn set_ckb_related_info(&mut self, info: &CkbRelatedInfo) -> ProtocolResult<()> {
        self.trie
            .insert(
                CKB_RELATED_INFO_KEY.as_bytes().to_vec(),
                info.encode()?.to_vec(),
            )
            .map_err(Into::into)
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

        if (metadata.version.start != epoch_segment.last_block_number() + 1) && metadata.epoch != 0
        {
            return Err(SystemScriptError::MetadataVersionDiscontinuity.into());
        }

        // Build propose counter
        let map = metadata
            .verifier_list
            .iter()
            .map(|v| (v.address, 0u64))
            .collect::<BTreeMap<_, _>>();
        let mut metadata = metadata.clone();
        metadata.propose_counter = map.into_iter().map(Into::into).collect();

        epoch_segment.append_endpoint(metadata.version.end)?;

        self.trie.insert(
            EPOCH_SEGMENT_KEY.as_bytes().to_vec(),
            epoch_segment.as_bytes(),
        )?;
        self.trie.insert(
            metadata.epoch.to_be_bytes().to_vec(),
            metadata.encode()?.to_vec(),
        )?;
        let new_root = self.trie.commit()?;
        CURRENT_METADATA_ROOT.with(|r| *r.borrow_mut() = new_root);

        Ok(())
    }

    pub fn update_propose_count(
        &mut self,
        block_number: u64,
        proposer: &H160,
    ) -> ProtocolResult<()> {
        let mut metadata = self.get_metadata_by_block_number(block_number)?;
        if let Some(counter) = metadata
            .propose_counter
            .iter_mut()
            .find(|p| &p.address == proposer)
        {
            counter.increase();
        }

        self.trie.insert(
            metadata.epoch.to_be_bytes().to_vec(),
            metadata.encode()?.to_vec(),
        )?;
        let new_root = self.trie.commit()?;
        CURRENT_METADATA_ROOT.with(|r| *r.borrow_mut() = new_root);

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
            .ok_or_else(|| SystemScriptError::MissingRecord(epoch))?;
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
            .ok_or_else(|| SystemScriptError::NoneCkbRelatedInfo)?;
        CkbRelatedInfo::decode(raw)
    }

    pub fn update_consensus_config(&mut self, config: ConsensusConfig) -> ProtocolResult<()> {
        let epoch_segment = self.get_epoch_segment()?;
        let latest_epoch = epoch_segment.get_latest_epoch_number();
        let mut metadata = self.get_metadata(latest_epoch)?;

        metadata.consensus_config = config.into();
        self.trie.insert(
            metadata.epoch.to_be_bytes().to_vec(),
            metadata.encode()?.to_vec(),
        )?;
        let new_root = self.trie.commit()?;
        CURRENT_METADATA_ROOT.with(|r| *r.borrow_mut() = new_root);
        Ok(())
    }

    fn get_epoch_by_block_number(&self, block_number: u64) -> ProtocolResult<u64> {
        self.get_epoch_segment()?.get_epoch_number(block_number)
    }
}
