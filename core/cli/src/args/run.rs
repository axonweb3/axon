use clap::Parser;

use common_config_parser::types::{spec::ChainSpec, Config};
use common_version::Version;
use core_run::{Axon, KeyProvider};

use crate::{
    error::{Error, Result},
    utils,
};

#[derive(Parser, Debug)]
#[command(about = "Run axon process")]
pub struct RunArgs {
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
}

impl RunArgs {
    pub(crate) fn execute<K: KeyProvider>(
        self,
        application_version: Version,
        kernel_version: Version,
        key_provider: Option<K>,
    ) -> Result<()> {
        let Self { config, spec } = self;
        let genesis = spec.genesis.build_rich_block();
        utils::check_version(
            &config.data_path_for_version(),
            &kernel_version,
            utils::latest_compatible_version(),
        )?;
        utils::register_log(&config);
        Axon::new(application_version.to_string(), config, spec, genesis)
            .run(key_provider)
            .map_err(Error::Running)
    }
}
