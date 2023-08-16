pub use core_executor::{DefaultFeeAllocator, FeeAllocate, FeeInlet};
pub use core_run::KeyProvider;
pub use protocol::{
    async_trait,
    types::{ValidatorExtend, H160, U256},
};

use std::sync::Arc;

use core_cli::{AxonCli, Result};
use core_executor::FEE_ALLOCATOR;

pub fn run(
    fee_allocator: impl FeeAllocate + 'static,
    key_provider: impl KeyProvider,
    app_version: &'static str,
) -> Result<()> {
    FEE_ALLOCATOR.swap(Arc::new(Box::new(fee_allocator)));

    AxonCli::init(
        clap::crate_version!()
            .parse()
            .expect("Parse kernel version"),
        app_version.parse().expect("Parse application version"),
    )
    .start_with_custom_key_provider(Some(key_provider))
}
