use std::sync::Arc;

use crossbeam::queue::ArrayQueue;

use crate::types::Requests;

pub struct CrossChainPipeline {
    queue: Arc<ArrayQueue<Requests>>,
}
