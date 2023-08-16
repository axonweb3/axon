use std::ffi::OsStr;

use clap::{builder::TypedValueParser, Parser};

use common_config_parser::types::{Config, JsonValueParser};
use common_version::Version;
use core_run::{Axon, KeyProvider};
use protocol::types::RichBlock;

use crate::{
    error::{Error, Result},
    utils,
};

#[derive(Parser, Debug)]
#[command(about = "Run axon process")]
pub struct RunArgs {
    #[arg(short = 'c', long = "config", help = "Axon config path")]
    pub config:  Config,
    #[arg(short = 'g', long = "genesis", help = "Axon genesis path")]
    #[arg(value_parser=RichBlockValueParser)]
    pub genesis: RichBlock,
}

#[derive(Clone, Debug)]
struct RichBlockValueParser;

impl TypedValueParser for RichBlockValueParser {
    type Value = RichBlock;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        JsonValueParser::<RichBlock>::default().parse_ref(cmd, arg, value)
    }
}

impl RunArgs {
    pub(crate) fn execute<K: KeyProvider>(
        self,
        application_version: Version,
        kernel_version: Version,
        key_provider: Option<K>,
    ) -> Result<()> {
        let Self { config, genesis } = self;
        utils::check_version(
            &config.data_path_for_version(),
            &kernel_version,
            utils::latest_compatible_version(),
        )?;
        utils::register_log(&config);
        Axon::new(application_version.to_string(), config, genesis)
            .run(key_provider)
            .map_err(Error::Running)
    }
}
