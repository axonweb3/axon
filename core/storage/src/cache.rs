use lru::LruCache;
use parking_lot::Mutex;

use protocol::types::{Block, Bytes, Hash, Header, Receipt, SignedTransaction};

const LRU_CAPACITY: usize = 200;

#[derive(Debug)]
pub struct StorageCache {
    pub blocks:        Mutex<LruCache<u64, Block>>,
    pub block_numbers: Mutex<LruCache<Hash, u64>>,
    pub headers:       Mutex<LruCache<u64, Header>>,
    pub transactions:  Mutex<LruCache<Hash, SignedTransaction>>,
    pub codes:         Mutex<LruCache<Hash, Bytes>>,
    pub receipts:      Mutex<LruCache<Hash, Receipt>>,
}

impl Default for StorageCache {
    fn default() -> Self {
        StorageCache {
            blocks:        Mutex::new(LruCache::new(LRU_CAPACITY)),
            block_numbers: Mutex::new(LruCache::new(LRU_CAPACITY)),
            headers:       Mutex::new(LruCache::new(LRU_CAPACITY)),
            transactions:  Mutex::new(LruCache::new(LRU_CAPACITY)),
            codes:         Mutex::new(LruCache::new(LRU_CAPACITY)),
            receipts:      Mutex::new(LruCache::new(LRU_CAPACITY)),
        }
    }
}
