mod backend;
mod trie;

pub use backend::{apply::AxonExecutorApplyAdapter, read_only::AxonExecutorReadOnlyAdapter};
pub use trie::{db::RocksTrieDB, wrapped::MPTTrie};

#[macro_export]
macro_rules! blocking_async {
    ($self_: ident, $adapter: ident, $method: ident$ (, $args: expr)*) => {{
        let rt = protocol::tokio::runtime::Handle::current();
        let adapter_clone = $self_.$adapter();

        protocol::tokio::task::block_in_place(move || {
            rt.block_on(adapter_clone.$method( $($args,)* )).unwrap()
        })
    }};
}
