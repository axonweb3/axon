mod native_token;

pub use crate::system_contract::native_token::NativeTokenContract;

use protocol::traits::{ApplyBackend, Backend};
use protocol::types::{SignedTransaction, TxResp, H160};

pub const fn system_contract_address(addr: u8) -> H160 {
    H160([
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, addr,
    ])
}

pub trait SystemContract {
    const ADDRESS: H160;

    fn exec_<B: Backend + ApplyBackend>(&self, backend: &mut B, tx: &SignedTransaction) -> TxResp;
}

pub fn system_contract_dispatch<B: Backend + ApplyBackend>(
    backend: &mut B,
    tx: &SignedTransaction,
) -> Option<TxResp> {
    let native_token_address = NativeTokenContract::ADDRESS;
    if let Some(addr) = tx.get_to() {
        if addr == native_token_address {
            return Some(NativeTokenContract::default().exec_(backend, tx));
        }
    }

    None
}
