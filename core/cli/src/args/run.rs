use clap::Parser;

use common_config_parser::types::Config;
use common_version::Version;
use core_run::KeyProvider;

use crate::{
    error::{Error, Result},
    utils,
};

#[derive(Parser, Debug)]
#[command(about = "Run axon service")]
pub struct RunArgs {
    #[arg(
        short = 'c',
        long = "config",
        value_name = "CONFIG_FILE",
        help = "File path of client configurations."
    )]
    pub config: Config,
}

impl RunArgs {
    pub(crate) fn execute<K: KeyProvider>(
        self,
        application_version: Version,
        kernel_version: Version,
        key_provider: Option<K>,
    ) -> Result<()> {
        let Self { config } = self;

        utils::check_version(
            &config.data_path_for_version(),
            &kernel_version,
            utils::latest_compatible_version(),
        )?;
        utils::register_log(&config);

        let version = application_version.to_string();
        core_run::run(version, config, key_provider).map_err(Error::Running)
    }
}
