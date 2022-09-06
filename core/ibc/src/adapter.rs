use std::sync::Arc;

use protocol::traits::IbcCrossChainStorage;

pub struct IbcAdapter<S> {
    storage: Arc<S>,
}

impl<S> IbcAdapter<S>
where
    S: IbcCrossChainStorage + 'static,
{
    pub fn new(storage: Arc<S>) -> Self {
        IbcAdapter { storage }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config_parser::types::ConfigRocksDB;
    use core_storage::{adapter::rocks::RocksAdapter, ImplStorage};
    use std::path::PathBuf;

    #[test]
    fn example() {
        let path_block = PathBuf::new();
        let rocksdb = ConfigRocksDB::default();
        let mock_rocks_adapter =
            Arc::new(RocksAdapter::new(path_block.clone(), rocksdb.clone()).unwrap());

        let storage = Arc::new(ImplStorage::new(mock_rocks_adapter, 88));
        let ibc_adapter = IbcAdapter::new(storage);

        // pub struct GrpcService<S> {
        //     // IbcImpl
        //     // ibc: IbcImpl<GrpcAdapter, IbcRouter>,
        //     // adapter: Arc<RwLock<GrpcAdapter>>,
        //     store: Arc<S>,
        // }
        // GrpcAdapter
        // pub struct GrpcStorage {

        // }

        // impl<S> GrpcService<S>
        // where
        //     S: IbcCrossChainStorage,
        // {
        //     pub fn new() -> Self {
        //         GrpcService {
        //             // adapter: Arc::new(RwLock::new(GrpcAdapter{}))
        //             // ibc: IbcImpl::new(),
        //             store: Arc::new(storage),
        //         }
        //     }

        //     pub async fn run(&self) {
        //         todo!()
        //     }
        // }
    }
}
