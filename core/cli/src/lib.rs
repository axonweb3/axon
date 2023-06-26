use std::io::{self, Write};
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

        check_version(
            &config.data_path_for_version(),
            &self.version,
            &latest_compatible_version(),
        );
    }
}

fn check_version(p: &Path, current: &Version, least_compatible: &Version) {
    let ver_str = match std::fs::read_to_string(p) {
        Ok(x) => x,
        Err(e) if e.kind() == io::ErrorKind::NotFound => "".into(),
        Err(e) => panic!("failed to read version: {e}"),
    };

    if ver_str.is_empty() {
        return atomic_write(p, current.to_string().as_bytes()).unwrap();
    }

    let prev_version = Version::parse(&ver_str).unwrap();
    if prev_version < *least_compatible {
        panic!(
            "The previous version {} is not compatible with the current version {}",
            prev_version, current
        );
    }
    atomic_write(p, current.to_string().as_bytes()).unwrap();
}

/// Write content to p atomically. Create the parent directory if it doesn't
/// already exist.
///
/// # Panics
///
/// if p.parent() is None.
fn atomic_write(p: &Path, content: &[u8]) -> std::io::Result<()> {
    let parent = p.parent().unwrap();

    std::fs::create_dir_all(parent)?;

    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.as_file_mut().write_all(content)?;
    // https://stackoverflow.com/questions/7433057/is-rename-without-fsync-safe
    tmp.as_file_mut().sync_all()?;
    tmp.persist(p)?;
    let parent = std::fs::OpenOptions::new().read(true).open(parent)?;
    parent.sync_all()?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn test_check_version() {
        let tmp = NamedTempFile::new().unwrap();
        let p = tmp.path();
        // We just want NamedTempFile to delete the file on drop. We want to
        // start with the file not exist.
        std::fs::remove_file(p).unwrap();

        let least_compatible = "0.1.0-alpha.9".parse().unwrap();

        check_version(p, &"0.1.15".parse().unwrap(), &least_compatible);
        assert_eq!(std::fs::read_to_string(p).unwrap(), "0.1.15");

        check_version(p, &"0.2.0".parse().unwrap(), &least_compatible);
        assert_eq!(std::fs::read_to_string(p).unwrap(), "0.2.0");
    }

    #[should_panic = "The previous version"]
    #[test]
    fn test_check_version_failure() {
        let tmp = NamedTempFile::new().unwrap();
        let p = tmp.path();
        check_version(p, &"0.1.0".parse().unwrap(), &"0.1.0".parse().unwrap());
        check_version(p, &"0.2.0".parse().unwrap(), &"0.2.0".parse().unwrap());
    }
}
