use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use clap::builder::{IntoResettable, Str};
use clap::{Arg, ArgMatches, Command};
use semver::Version;

use common_config_parser::{parse_file, types::Config};
use core_run::{Axon, KeyProvider, SecioKeyPair};
use protocol::types::RichBlock;

pub struct AxonCli {
    version: Version,
    matches: ArgMatches,
}

impl AxonCli {
    pub fn init(ver: impl IntoResettable<Str>) -> Self {
        let matches = Command::new("axon")
            .version(ver)
            .arg(
                Arg::new("config_path")
                    .short('c')
                    .long("config")
                    .help("Axon config path")
                    .required(true)
                    .num_args(1),
            )
            .arg(
                Arg::new("genesis_path")
                    .short('g')
                    .long("genesis")
                    .help("Axon genesis path")
                    .required(true)
                    .num_args(1),
            )
            .subcommand(Command::new("run").about("Run axon process"));

        AxonCli {
            version: Version::parse(matches.get_version().unwrap()).unwrap(),
            matches: matches.get_matches(),
        }
    }

    pub fn start(&self) {
        self.start_with_custom_key_provider::<SecioKeyPair>(None)
    }

    pub fn start_with_custom_key_provider<K: KeyProvider>(&self, key_provider: Option<K>) {
        let config_path = self.matches.get_one::<String>("config_path").unwrap();
        let path = Path::new(&config_path).parent().unwrap();
        let mut config: Config = parse_file(config_path, false).unwrap();

        if let Some(ref mut f) = config.rocksdb.options_file {
            *f = path.join(&f)
        }
        let genesis: RichBlock = parse_file(
            self.matches.get_one::<String>("genesis_path").unwrap(),
            true,
        )
        .unwrap();

        self.check_version(&config);

        register_log(&config);

        Axon::new(config, genesis).run(key_provider).unwrap();
    }

    fn check_version(&self, config: &Config) {
        if !config.data_path.exists() {
            std::fs::create_dir_all(&config.data_path).unwrap();
        }

        let f_path = config.data_path_for_version();
        let mut f = File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(f_path)
            .unwrap();

        let mut ver_str = String::new();
        f.read_to_string(&mut ver_str).unwrap();

        if ver_str.is_empty() {
            return f.write_all(self.version.to_string().as_bytes()).unwrap();
        }

        let prev_version = Version::parse(&ver_str).unwrap();
        if prev_version < latest_compatible_version() {
            println!(
                "The previous version {:?} is not compatible with the current version {:?}",
                prev_version, self.version
            );
            std::process::exit(0);
        }

        f.seek(SeekFrom::Start(0)).unwrap();
        f.write_all(self.version.to_string().as_bytes()).unwrap();
        f.sync_all().unwrap();
    }
}

fn latest_compatible_version() -> Version {
    Version::parse("0.1.0-alpha.9").unwrap()
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
