pub use core_executor::{DefaultFeeAllocator, FeeAllocate, FeeInlet};
pub use protocol::types::{ValidatorExtend, H160, U256};

use std::sync::Arc;

use core_cli::AxonCli;
use core_executor::FEE_ALLOCATOR;

pub fn run(fee_allocator: impl FeeAllocate + 'static) {
    FEE_ALLOCATOR.swap(Arc::new(Box::new(fee_allocator)));
    AxonCli::init().start();
}
