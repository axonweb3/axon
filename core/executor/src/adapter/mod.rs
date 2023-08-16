mod apply;
mod read_only;
mod trie_db;
mod wrapped_trie;

pub use trie_db::RocksTrieDB;
pub use wrapped_trie::MPTTrie;
pub use {apply::AxonExecutorApplyAdapter, read_only::AxonExecutorReadOnlyAdapter};

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
