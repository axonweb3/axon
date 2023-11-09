use ckb_traits::{CellDataProvider, ExtensionProvider, HeaderProvider};
use ckb_types::core::cell::CellProvider;

use crate::types::{Bytes, CellDep, VMResp};
use crate::{traits::Context, ProtocolResult};

pub const BYTE_SHANNONS: u64 = 100_000_000;
pub const SIGNATURE_HASH_CELL_OCCUPIED_CAPACITY: u64 = signature_hash_cell_bytes() * BYTE_SHANNONS;

/// The always success cell structure:
/// ```yml
/// type:
///     Null
/// lock:
///     code_hash: H256::zero()
///     args: 0x
///     hash_type: data
/// data: signature hash(32 bytes)
/// capacity: 1b31d2900
/// ```
/// So the occupied bytes is 32 + 32 + 1 + 8 = 73 bytes.
const fn signature_hash_cell_bytes() -> u64 {
    32 + 32 + 1 + 8
}

pub trait CkbDataProvider:
    Clone + CellDataProvider + CellProvider + HeaderProvider + ExtensionProvider
{
}

pub trait Interoperation: Sync + Send {
    fn call_ckb_vm<DL: CellDataProvider>(
        ctx: Context,
        data_loader: &DL,
        data_cell_dep: CellDep,
        args: &[Bytes],
        max_cycles: u64,
    ) -> ProtocolResult<VMResp>;
}
