use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::{cell::CellProvider, Cycle, TransactionView};
use ckb_types::{packed, prelude::*};

use crate::lazy::{ALWAYS_SUCCESS_TYPE_SCRIPT, DUMMY_INPUT_OUT_POINT};
use crate::types::{Bytes, CellDep, CellWithData, SignatureR, SignatureS, VMResp};
use crate::{traits::Context, ProtocolResult};

pub const BYTE_SHANNONS: u64 = 100_000_000;
pub const ALWAYS_SUCCESS_CELL_OCCUPIED_CAPACITY: u64 = always_success_cell_bytes() * BYTE_SHANNONS;

/// The always success cell structure:
/// ```yml
/// type:
///     code_hash: ckb_blake2b_256(ALWAYS_SUCCESS)
///     args: axon_transaction.sig_hash(with_chain_id)
///     hash_type: data1
/// lock:
///     code_hash: H256::zero()
///     args: 0x
///     hash_type: data
/// data: 0x
/// capacity: 0x277cf2a00  
/// ```
/// So the occupied bytes is 32 + 32 + 1 + 32 + 1 + 8 = 106 bytes.
const fn always_success_cell_bytes() -> u64 {
    32 + 32 + 1 + 32 + 1 + 8
}

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
        data_loader: &DL,
        mocked_tx: &TransactionView,
        dummy_input: Option<CellWithData>,
        max_cycles: u64,
    ) -> ProtocolResult<Cycle>;

    /// The function construct the `TransactionView` payload required by
    /// `verify_by_ckb_vm()`.
    fn dummy_transaction(
        r: SignatureR,
        s: SignatureS,
        signature_hash: Option<[u8; 32]>,
    ) -> TransactionView {
        let cell_deps = r.cell_deps();
        let header_deps = r.header_deps();
        let signature_hash = signature_hash.map(|hash| hash.to_vec()).unwrap_or_default();

        let tx_builder = TransactionView::new_advanced_builder()
            .cell_deps(cell_deps.iter().map(Into::into))
            .header_deps(header_deps.iter().map(|dep| dep.0.pack()))
            .witnesses(s.witnesses.iter().map(|i| {
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
            }));

        if r.is_only_by_ref() {
            return tx_builder
                .inputs(r.out_points().iter().map(|i| {
                    packed::CellInput::new(
                        packed::OutPointBuilder::default()
                            .tx_hash(i.tx_hash.0.pack())
                            .index(i.index.pack())
                            .build(),
                        0u64,
                    )
                }))
                .output(
                    packed::CellOutputBuilder::default()
                        .type_(
                            Some(
                                ALWAYS_SUCCESS_TYPE_SCRIPT
                                    .clone()
                                    .as_builder()
                                    .args(signature_hash.pack())
                                    .build(),
                            )
                            .pack(),
                        )
                        .capacity(ALWAYS_SUCCESS_CELL_OCCUPIED_CAPACITY.pack())
                        .build(),
                )
                .build();
        }

        let output_capacity = (r.dummy_input().unwrap().capacity() - BYTE_SHANNONS)
            .max(ALWAYS_SUCCESS_CELL_OCCUPIED_CAPACITY);

        tx_builder
            .input(packed::CellInput::new(DUMMY_INPUT_OUT_POINT.clone(), 0u64))
            .output(
                packed::CellOutputBuilder::default()
                    .type_(
                        Some(
                            ALWAYS_SUCCESS_TYPE_SCRIPT
                                .clone()
                                .as_builder()
                                .args(signature_hash.pack())
                                .build(),
                        )
                        .pack(),
                    )
                    .capacity(output_capacity.pack())
                    .build(),
            )
            .build()
    }
}
