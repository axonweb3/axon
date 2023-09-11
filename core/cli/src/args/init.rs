use clap::Parser;

use common_config_parser::types::{
    spec::{ChainSpec, PrivateKey},
    Config,
};
use common_version::Version;

use crate::{
    error::{Error, Result},
    utils,
};

#[derive(Parser, Debug)]
#[command(about = "Initialize new axon data directory")]
pub struct InitArgs {
    #[arg(
        short = 'c',
        long = "config",
        value_name = "CONFIG_FILE",
        help = "File path of client configurations."
    )]
    pub config: Config,
    #[arg(
        short = 's',
        long = "chain-spec",
        value_name = "CHAIN_SPEC_FILE",
        help = "File path of chain spec."
    )]
    pub spec:   ChainSpec,

    #[command(flatten)]
    pub key: PrivateKey,
}

impl InitArgs {
    pub(crate) fn execute(self, kernel_version: Version) -> Result<()> {
        let Self { config, spec, key } = self;

        let key_data = key.data().map_err(Error::Internal)?;

        utils::check_version(
            &config.data_path_for_version(),
            &kernel_version,
            utils::latest_compatible_version(),
        )?;
        utils::register_log(&config);

        core_run::init(config, spec, key_data).map_err(Error::Running)
    }
}
