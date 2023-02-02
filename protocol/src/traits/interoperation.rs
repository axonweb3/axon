use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::{cell::CellProvider, Cycle, TransactionView};
use ckb_types::{packed, prelude::*};

use crate::types::{Bytes, CellDep, OutPoint, VMResp, Witness, H256};
use crate::{traits::Context, ProtocolResult};

pub trait Interoperation: Sync + Send {
    fn call_ckb_vm<DL: CellDataProvider>(
        ctx: Context,
        data_loader: &DL,
        data_cell_dep: CellDep,
        args: &[Bytes],
        max_cycles: u64,
    ) -> ProtocolResult<VMResp>;

    fn verify_by_ckb_vm<DL: CellProvider + CellDataProvider + HeaderProvider>(
        ctx: Context,
        data_loader: DL,
        mocked_tx: &TransactionView,
        max_cycles: u64,
    ) -> ProtocolResult<Cycle>;

    /// The function construct the `TransactionView` payload required by
    /// `verify_by_ckb_vm`.
    fn dummy_transaction(
        cell_deps: Vec<CellDep>,
        header_deps: Vec<H256>,
        inputs: Vec<OutPoint>,
        witnesses: Vec<Witness>,
    ) -> TransactionView {
        TransactionView::new_advanced_builder()
            .inputs(inputs.iter().map(|i| {
                packed::CellInput::new(
                    packed::OutPointBuilder::default()
                        .tx_hash(i.tx_hash.0.pack())
                        .index(i.index.pack())
                        .build(),
                    0u64,
                )
            }))
            .witnesses(witnesses.iter().map(|i| {
                packed::WitnessArgsBuilder::default()
                    .input_type(
                        packed::BytesOptBuilder::default()
                            .set(i.input_type.clone().map(|inner| inner.pack()))
                            .build(),
                    )
                    .output_type(
                        packed::BytesOptBuilder::default()
                            .set(i.output_type.clone().map(|inner| inner.pack()))
                            .build(),
                    )
                    .lock(
                        packed::BytesOptBuilder::default()
                            .set(i.lock.clone().map(|inner| inner.pack()))
                            .build(),
                    )
                    .build()
                    .as_bytes()
                    .pack()
            }))
            .cell_deps(cell_deps.into_iter().map(Into::into))
            .header_deps(header_deps.iter().map(|dep| dep.0.pack()))
            .build()
    }
}
