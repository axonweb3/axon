use clap::{crate_version, App, Arg, ArgMatches, SubCommand};

use common_config_parser::{parse_file, types::Config};
use core_run::Axon;
use protocol::types::{Genesis, Metadata};

pub struct AxonCli<'a> {
    matches: ArgMatches<'a>,
}

impl<'a> AxonCli<'a> {
    pub fn init() -> Self {
        let matches = App::new("axon")
            .version(crate_version!())
            .arg(
                Arg::with_name("config_path")
                    .short("c")
                    .long("config")
                    .help("Axon config path")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("genesis_path")
                    .short("g")
                    .long("genesis")
                    .help("Axon genesis path")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("metadata_path")
                    .short("m")
                    .long("metadata")
                    .help("Axon metadata path")
                    .required(true)
                    .takes_value(true),
            )
            .subcommand(SubCommand::with_name("run").about("Run axon process"))
            .get_matches();

        AxonCli { matches }
    }

    pub fn start(&self) {
        let config: Config =
            parse_file(self.matches.value_of("config_path").unwrap(), false).unwrap();
        let genesis: Genesis =
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
