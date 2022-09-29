use std::sync::Arc;

use once_cell::sync::OnceCell;

use protocol::traits::IbcAdapter;

pub static IBC_HANDLER: OnceCell<Box<dyn IbcHandle>> = OnceCell::new();

pub trait IbcHandle: Send + Sync {
    fn handle(&self, raw: &[u8]) -> u8;
}

pub struct IbcHandlerImpl<Adapter> {
    _adapter: Arc<Adapter>,
}

impl<Adapter: IbcAdapter + 'static> IbcHandle for IbcHandlerImpl<Adapter> {
    fn handle(&self, _raw: &[u8]) -> u8 {
        todo!()
        // let msg = raw.decode();
        // self.adapter.get();
        // self.adapter.set();
    }
}

impl<Adapter: IbcAdapter + 'static> IbcHandlerImpl<Adapter> {
    pub fn new(_adapter: Arc<Adapter>) -> Self {
        IbcHandlerImpl { _adapter }
    }
}
