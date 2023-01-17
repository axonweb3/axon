use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::{Cycle, TransactionView};
use ckb_types::packed;

use crate::types::{Bytes, VMResp};
use crate::{traits::Context, ProtocolResult};

pub trait Interoperation: Sync + Send {
    fn call_ckb_vm<'a, DL: CellDataProvider>(
        ctx: Context,
        data_loader: &'a DL,
        data_cell_dep: packed::CellDep,
        args: &[Bytes],
        max_cycles: u64,
    ) -> ProtocolResult<VMResp>;

    fn verify_by_ckb_vm<'a, DL: CellDataProvider + HeaderProvider>(
        ctx: Context,
        data_loader: &'a DL,
        mocked_tx: &'a TransactionView,
        max_cycles: u64,
    ) -> ProtocolResult<Cycle>;
}
