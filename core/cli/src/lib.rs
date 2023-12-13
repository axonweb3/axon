mod args;
mod error;
pub(crate) mod utils;

pub use args::{
    generate_keypair::GenerateKeypairArgs, hardfork::HardforkArgs, init::InitArgs,
    recover_keypair::RecoverKeypairArgs, run::RunArgs,
};
pub use error::{CheckingVersionError, Error, Result};

use clap::{CommandFactory as _, FromArgMatches as _, Parser, Subcommand};

use common_version::Version;
use core_run::{KeyProvider, SecioKeyPair};

#[derive(Parser, Debug)]
#[command(name = "axon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init(InitArgs),
    Run(RunArgs),
    Hardfork(HardforkArgs),
    GenerateKeypair(GenerateKeypairArgs),
    RecoverKeypair(RecoverKeypairArgs),
}

pub struct AxonCli {
    application_version: Version,
    kernel_version:      Version,
    inner:               Cli,
}

impl AxonCli {
    pub fn init(application_version: Version, kernel_version: Version) -> Self {
        let mix_version = format!(
            "{}-with-axon-kernel-{}",
            application_version, kernel_version
        );
        let cmd = Cli::command().version(mix_version);
        let cli = Cli::from_arg_matches(&cmd.get_matches()).unwrap_or_else(|_| unreachable!());
        AxonCli {
            kernel_version,
            application_version,
            inner: cli,
        }
    }

    pub fn start(self) -> Result<()> {
        self.start_with_custom_key_provider::<SecioKeyPair>(None)
    }

    pub fn start_with_custom_key_provider<K: KeyProvider>(
        self,
        key_provider: Option<K>,
    ) -> Result<()> {
        let AxonCli {
            application_version,
            kernel_version,
            inner: cli,
        } = self;
        match cli.command {
            Commands::Init(args) => args.execute(kernel_version),
            Commands::Run(args) => args.execute(application_version, kernel_version, key_provider),
            Commands::Hardfork(args) => args.execute(),
            Commands::GenerateKeypair(args) => args.execute(),
            Commands::RecoverKeypair(args) => args.execute(),
        }
    }
}
