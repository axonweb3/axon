use ibc_proto::google::protobuf::Any;
use tendermint_abci::Application;
use tendermint_proto::{
    abci::{
        Event, RequestBeginBlock, RequestDeliverTx, RequestInfo, RequestInitChain, RequestQuery,
        ResponseBeginBlock, ResponseCommit, ResponseDeliverTx, ResponseInfo, ResponseInitChain,
        ResponseQuery,
    },
};

/// BaseCoin ABCI application.
///
/// Can be safely cloned and sent across threads, but not shared.
#[derive(Clone)]
pub(crate) struct BaseCoinApp {
    // store: MainStore<S>,
    // modules: SharedRw<ModuleList<S>>,
}

impl BaseCoinApp {
    pub fn new() -> Self {
        BaseCoinApp {  }
    }
    // try to deliver the message to all registered modules
    // if `module.deliver()` returns `Error::not_handled()`, try next module
    // Return:
    // * other errors immediately OR
    // * `Error::not_handled()` if all modules return `Error::not_handled()`
    // * events from first successful deliver call
    fn deliver_msg(&self, message: Any) {
        todo!()
    }
}

impl Application for BaseCoinApp {
    fn info(&self, request: RequestInfo) -> ResponseInfo {
        todo!()
    }

    fn init_chain(&self, request: RequestInitChain) -> ResponseInitChain {
        todo!()
    }

    fn query(&self, request: RequestQuery) -> ResponseQuery {
        todo!()
    }

    fn deliver_tx(&self, request: RequestDeliverTx) -> ResponseDeliverTx {
        todo!()
    }

    fn commit(&self) -> ResponseCommit {
        todo!()
    }

    fn begin_block(&self, request: RequestBeginBlock) -> ResponseBeginBlock {
        todo!()
    }
}
