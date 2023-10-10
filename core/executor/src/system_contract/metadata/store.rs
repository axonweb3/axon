use std::collections::BTreeMap;
use std::sync::Arc;

use common_config_parser::types::spec::HardforkName;
use protocol::trie::Trie as _;
use protocol::types::{
    CkbRelatedInfo, ConsensusConfig, ConsensusConfigV0, HardforkInfo, HardforkInfoInner, Metadata,
    MetadataInner, H160, H256,
};
use protocol::{codec::ProtocolCodec, ProtocolResult};

use crate::system_contract::metadata::{
    segment::EpochSegment, CKB_RELATED_INFO_KEY, CONSENSUS_CONFIG, EPOCH_SEGMENT_KEY,
    HARDFORK_INFO, HARDFORK_KEY,
};
use crate::system_contract::{error::SystemScriptError, METADATA_DB};
use crate::{adapter::RocksTrieDB, MPTTrie, CURRENT_METADATA_ROOT};

/// The metadata store does not follow the storage layout of EVM smart contract.
/// It use MPT called Metadata MPT with the following layout:
/// | key                  | value                    |
/// | -------------------- | ------------------------ |
/// | EPOCH_SEGMENT_KEY    | `EpochSegment.encode()`  |
/// | CKB_RELATED_INFO_KEY | `CkbRelatedInfo.encode()`|
/// | HARDFORK_KEY         | `HardforkInfo.encode()`  |
/// | epoch_0.be_bytes()   | `Metadata.encode()`      |
/// | epoch_1.be_bytes()   | `Metadata.encode()`      |
/// | ...                  | ...                      |
///
/// All these data are stored in a the `c9` column family of RocksDB, and the
/// root of the Metadata MPT is stored in the storage MPT of the metadata
/// contract Account as follow:
///
/// **Metadata Account**
/// | address | `0xFFfffFFfFFfffFfFffFFfFfFfFffFfffFFFFFf01`|
/// | nonce   | `0x0`                                       |
/// | balance | `0x0`                                       |
/// | storage | `storage_root`                              |
///
/// **Metadata Storage MPT**
/// | METADATA_ROOT_KEY | Metadata MPT root |
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

        let (inner, config) = metadata.into_part();
        let current_hardfork = **HARDFORK_INFO.load();

        self.trie.insert(
            EPOCH_SEGMENT_KEY.as_bytes().to_vec(),
            epoch_segment.as_bytes(),
        )?;
        self.trie
            .insert(inner.epoch.to_be_bytes().to_vec(), inner.encode()?.to_vec())?;
        let config = encode_consensus_config(current_hardfork, config.encode()?.to_vec());
        self.trie
            .insert(CONSENSUS_CONFIG.as_bytes().to_vec(), config)?;
        let new_root = self.trie.commit()?;
        CURRENT_METADATA_ROOT.with(|r| *r.borrow_mut() = new_root);

        Ok(())
    }

    pub fn update_propose_count(
        &mut self,
        block_number: u64,
        proposer: &H160,
    ) -> ProtocolResult<()> {
        let epoch = self.get_epoch_by_block_number(block_number)?;
        let mut metadata = self.get_metadata_inner(epoch)?;
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
        let inner = self.get_metadata_inner(epoch)?;
        let config = self.get_consensus_config()?;
        Ok(Metadata::from_parts(inner, config))
    }

    fn get_metadata_inner(&self, epoch: u64) -> ProtocolResult<MetadataInner> {
        let raw = self
            .trie
            .get(&epoch.to_be_bytes())?
            .ok_or_else(|| SystemScriptError::MissingRecord(epoch))?;
        MetadataInner::decode(raw)
    }

    pub fn get_consensus_config(&self) -> ProtocolResult<ConsensusConfig> {
        let raw = self
            .trie
            .get(CONSENSUS_CONFIG.as_bytes())?
            .expect("Inner panic with can't find consensus config");

        decode_consensus_config(raw)
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
        let current_hardfork = **HARDFORK_INFO.load();
        self.trie.insert(
            CONSENSUS_CONFIG.as_bytes().to_vec(),
            encode_consensus_config(current_hardfork, config.encode()?.to_vec()),
        )?;
        let new_root = self.trie.commit()?;
        CURRENT_METADATA_ROOT.with(|r| *r.borrow_mut() = new_root);
        Ok(())
    }

    fn get_epoch_by_block_number(&self, block_number: u64) -> ProtocolResult<u64> {
        self.get_epoch_segment()?.get_epoch_number(block_number)
    }

    pub fn set_hardfork_info(&mut self, block_number: u64, info: H256) -> ProtocolResult<()> {
        let current_info = {
            match self.trie.get(HARDFORK_KEY.as_bytes())? {
                Some(data) => {
                    let mut hardfork_info = HardforkInfo::decode(data)?;

                    hardfork_info.push(HardforkInfoInner {
                        block_number,
                        flags: info,
                    });
                    hardfork_info
                }
                None => HardforkInfo {
                    inner: vec![HardforkInfoInner {
                        block_number,
                        flags: info,
                    }],
                },
            }
        };

        self.trie.insert(
            HARDFORK_KEY.as_bytes().to_vec(),
            current_info.encode()?.to_vec(),
        )?;
        let new_root = self.trie.commit()?;
        CURRENT_METADATA_ROOT.with(|r| *r.borrow_mut() = new_root);
        Ok(())
    }

    pub fn hardfork_info(&self, target_number: u64) -> ProtocolResult<H256> {
        match self.trie.get(HARDFORK_KEY.as_bytes())? {
            Some(data) => {
                let hardfork_info = HardforkInfo::decode(data)?;

                for k in hardfork_info.inner.iter().rev() {
                    if k.block_number > target_number {
                        continue;
                    }
                    return Ok(k.flags);
                }
                Ok(H256::zero())
            }
            None => Ok(H256::zero()),
        }
    }

    pub fn hardfork_infos(&self) -> ProtocolResult<HardforkInfo> {
        match self.trie.get(HARDFORK_KEY.as_bytes())? {
            Some(data) => HardforkInfo::decode(data),
            None => Ok(HardforkInfo::default()),
        }
    }
}

#[derive(Debug)]
enum ConsensusConfigFlag {
    V0 = 0b0,
    V1 = 0b1,
}

impl From<u16> for ConsensusConfigFlag {
    fn from(value: u16) -> Self {
        match value {
            0b0 => ConsensusConfigFlag::V0,
            0b1 => ConsensusConfigFlag::V1,
            _ => unreachable!(),
        }
    }
}

impl ConsensusConfigFlag {
    fn new(flags: H256) -> Self {
        let v1_name_flag = H256::from_low_u64_be((HardforkName::Andromeda as u64).to_be());
        let res = flags & v1_name_flag;

        if res & v1_name_flag == v1_name_flag {
            ConsensusConfigFlag::V1
        } else {
            ConsensusConfigFlag::V0
        }
    }
}

fn decode_consensus_config(raw: Vec<u8>) -> ProtocolResult<ConsensusConfig> {
    let raw_flag = {
        let mut a = [0u8; 2];
        a[0] = raw[0];
        a[1] = raw[1];
        a
    };
    let flag = ConsensusConfigFlag::from(u16::from_be_bytes(raw_flag));

    match flag {
        ConsensusConfigFlag::V0 => ConsensusConfigV0::decode(&raw[2..]).map(Into::into),
        ConsensusConfigFlag::V1 => ConsensusConfig::decode(&raw[2..]),
    }
}

pub fn encode_consensus_config(current_hardfork: H256, config: Vec<u8>) -> Vec<u8> {
    let flag = ConsensusConfigFlag::new(current_hardfork);

    let mut res = (flag as u16).to_be_bytes().to_vec();
    res.extend(config);
    res
}
