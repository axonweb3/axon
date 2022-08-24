mod adapter;
mod client;
mod error;
mod grpc;
mod transfer;

use std::sync::Arc;

pub struct IbcImpl<Adapter> {
    _adapter: Arc<Adapter>,
}
