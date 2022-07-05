use lru::LruCache;
use parking_lot::Mutex;

use protocol::types::{Block, Bytes, Hash, Header, Receipt, SignedTransaction};

#[derive(Debug)]
pub struct StorageCache {
    pub blocks:        Mutex<LruCache<u64, Block>>,
    pub block_numbers: Mutex<LruCache<Hash, u64>>,
    pub headers:       Mutex<LruCache<u64, Header>>,
    pub transactions:  Mutex<LruCache<Hash, SignedTransaction>>,
    pub codes:         Mutex<LruCache<Hash, Bytes>>,
    pub receipts:      Mutex<LruCache<Hash, Receipt>>,
}

impl StorageCache {
    pub fn new(size: usize) -> Self {
        StorageCache {
            blocks:        Mutex::new(LruCache::new(size)),
            block_numbers: Mutex::new(LruCache::new(size)),
            headers:       Mutex::new(LruCache::new(size)),
            transactions:  Mutex::new(LruCache::new(size)),
            codes:         Mutex::new(LruCache::new(size)),
            receipts:      Mutex::new(LruCache::new(size)),
        }
    }
}
