pub use ckb_crosschain_schema::{CkbCrossChainSchema, MonitorCkbNumberSchema};

use protocol::traits::{StorageCategory, StorageSchema};
use protocol::types::{
    Block, Bytes, DBBytes, Hash, HashWithDirection, Header, Proof, Receipt, SignedTransaction,
};

use crate::hash_key::{BlockKey, CommonHashKey};

macro_rules! impl_storage_schema_for {
    ($name: ident, $key: ty, $val: ty, $category: ident) => {
        pub struct $name;

        impl StorageSchema for $name {
            type Key = $key;
            type Value = $val;

            fn category() -> StorageCategory {
                StorageCategory::$category
            }
        }
    };
}

impl_storage_schema_for!(
    TransactionSchema,
    CommonHashKey,
    SignedTransaction,
    SignedTransaction
);
impl_storage_schema_for!(
    TransactionBytesSchema,
    CommonHashKey,
    DBBytes,
    SignedTransaction
);
impl_storage_schema_for!(BlockSchema, BlockKey, Block, Block);
impl_storage_schema_for!(BlockHeaderSchema, BlockKey, Header, BlockHeader);
impl_storage_schema_for!(BlockHashNumberSchema, Hash, u64, HashHeight);
impl_storage_schema_for!(ReceiptSchema, CommonHashKey, Receipt, Receipt);
impl_storage_schema_for!(ReceiptBytesSchema, CommonHashKey, DBBytes, Receipt);
impl_storage_schema_for!(TxHashNumberSchema, Hash, u64, HashHeight);
impl_storage_schema_for!(LatestBlockSchema, Hash, Block, Block);
impl_storage_schema_for!(LatestProofSchema, Hash, Proof, Block);
impl_storage_schema_for!(OverlordWalSchema, Hash, Bytes, Wal);
impl_storage_schema_for!(EvmCodeSchema, Hash, Bytes, Code);
impl_storage_schema_for!(EvmCodeAddressSchema, Hash, Hash, Code);

mod ckb_crosschain_schema {
    use super::*;

    impl_storage_schema_for!(CkbCrossChainSchema, Hash, HashWithDirection, CkbCrossChain);
    impl_storage_schema_for!(MonitorCkbNumberSchema, Hash, u64, CkbCrossChain);
}
