use clap::Parser;

use common_config_parser::types::{spec::HardforkInput, Config};

use crate::error::{Error, Result};

#[derive(Parser, Debug)]
#[command(about = "About Axon hardfork feature")]
pub struct HardforkArgs {
    #[arg(
        short = 'c',
        long = "config",
        value_name = "CONFIG_FILE",
        help = "File path of client configurations."
    )]
    pub config: Config,

    #[command(flatten)]
    hardforks: Option<HardforkInput>,
}

impl HardforkArgs {
    pub fn execute(self) -> Result<()> {
        core_run::set_hardfork_info(self.config, self.hardforks.map(Into::into))
            .map_err(Error::Running)?;
        Ok(())
    }
}
