use crate::types::{Header, Metadata, TxResp, H160};
use crate::{traits::Context, ProtocolResult};

pub trait MetadataControl: Sync + Send {
    fn calc_epoch(&self, block_number: u64) -> u64;

    fn need_change_metadata(&self, block_number: u64) -> bool;

    fn update_metadata(&self, ctx: Context, header: &Header) -> ProtocolResult<()>;

    fn get_metadata(&self, ctx: Context, header: &Header) -> ProtocolResult<Metadata>;

    fn get_metadata_unchecked(&self, ctx: Context, block_number: u64) -> Metadata;
}

pub trait MetadataControlAdapter: Sync + Send {
    fn call_evm(
        &self,
        ctx: Context,
        header: &Header,
        addr: H160,
        data: Vec<u8>,
    ) -> ProtocolResult<TxResp>;
}
