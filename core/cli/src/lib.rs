use clap::{crate_version, Arg, ArgMatches, Command};

use common_config_parser::{parse_file, types::Config};
use core_run::Axon;
use protocol::types::{Metadata, RichBlock};

pub struct AxonCli {
    matches: ArgMatches,
}

impl AxonCli {
    pub fn init() -> Self {
        let matches = Command::new("axon")
            .version(crate_version!())
            .arg(
                Arg::new("config_path")
                    .short('c')
                    .long("config")
                    .help("Axon config path")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::new("genesis_path")
                    .short('g')
                    .long("genesis")
                    .help("Axon genesis path")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::new("metadata_path")
                    .short('m')
                    .long("metadata")
                    .help("Axon metadata path")
                    .required(true)
                    .takes_value(true),
            )
            .subcommand(Command::new("run").about("Run axon process"))
            .get_matches();

        AxonCli { matches }
    }

    pub fn start(&self) {
        let config: Config =
            parse_file(self.matches.value_of("config_path").unwrap(), false).unwrap();
        let genesis: RichBlock =
            parse_file(self.matches.value_of("genesis_path").unwrap(), true).unwrap();
        let metadata: Metadata =
            parse_file(self.matches.value_of("metadata_path").unwrap(), true).unwrap();

        register_log(&config);

        Axon::new(config, genesis, metadata).run().unwrap();
    }
}

fn register_log(config: &Config) {
    common_logger::init(
        config.logger.filter.clone(),
        config.logger.log_to_console,
        config.logger.console_show_file_and_line,
        config.logger.log_to_file,
        config.logger.metrics,
        config.logger.log_path.clone(),
        config.logger.file_size_limit,
        config.logger.modules_level.clone(),
    );
}
