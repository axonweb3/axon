use std::io::{self, Write};
use std::path::Path;

use clap::{Arg, ArgMatches, Command};
use protocol::ProtocolError;
use thiserror::Error;

use common_config_parser::{parse_file, types::Config, ParseError};
use common_version::Version;
use core_run::{Axon, KeyProvider, SecioKeyPair};
use protocol::types::RichBlock;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum Error {
    // Boxing so the error type isn't too large (clippy::result-large-err).
    #[error(transparent)]
    CheckingVersion(Box<CheckingVersionError>),
    #[error("reading data version: {0}")]
    ReadingVersion(#[source] io::Error),
    #[error("writing data version: {0}")]
    WritingVersion(#[source] io::Error),

    #[error("parsing config: {0}")]
    ParsingConfig(#[source] ParseError),
    #[error("getting parent directory of config file")]
    GettingParent,
    #[error("parsing genesis: {0}")]
    ParsingGenesis(#[source] ParseError),
    #[error("unknown subcommand: {0}")]
    UnknownSubcommand(String),

    #[error(transparent)]
    Running(ProtocolError),
}

#[non_exhaustive]
#[derive(Error, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[error("data version({data}) is not compatible with the current axon version({current}), version >= {least_compatible} is supported")]
pub struct CheckingVersionError {
    pub current:          Version,
    pub data:             Version,
    pub least_compatible: Version,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct AxonCli {
    kernel_version:      Version,
    application_version: Version,
    matches:             ArgMatches,
}

impl AxonCli {
    pub fn init(application_version: Version, kernel_version: Version) -> Self {
        let mix_version = format!(
            "{}-with-axon-kernel-{}",
            application_version, kernel_version
        );

        let matches = Command::new("axon")
            .version(mix_version)
            .subcommand_required(true)
            .subcommand(
                Command::new("run")
                    .about("Run axon process")
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
                    ),
            );

        AxonCli {
            kernel_version,
            application_version,
            matches: matches.get_matches(),
        }
    }

    pub fn start(&self) -> Result<()> {
        self.start_with_custom_key_provider::<SecioKeyPair>(None)
    }

    pub fn start_with_custom_key_provider<K: KeyProvider>(
        &self,
        key_provider: Option<K>,
    ) -> Result<()> {
        if let Some((cmd, matches)) = self.matches.subcommand() {
            match cmd {
                "run" => {
                    let config_path = matches.get_one::<String>("config_path").unwrap();
                    let path = Path::new(&config_path)
                        .parent()
                        .ok_or(Error::GettingParent)?;
                    let mut config: Config =
                        parse_file(config_path, false).map_err(Error::ParsingConfig)?;

                    if let Some(ref mut f) = config.rocksdb.options_file {
                        *f = path.join(&f)
                    }
                    let genesis: RichBlock =
                        parse_file(matches.get_one::<String>("genesis_path").unwrap(), true)
                            .map_err(Error::ParsingGenesis)?;

                    self.check_version(&config)?;

                    register_log(&config);

                    Axon::new(self.application_version.to_string(), config, genesis)
                        .run(key_provider)
                        .map_err(Error::Running)
                }
                _ => Err(Error::UnknownSubcommand(cmd.to_owned())),
            }
        } else {
            // Since `clap.subcommand_required(true)`.
            unreachable!();
        }
    }

    fn check_version(&self, config: &Config) -> Result<()> {
        // Won't panic because parent of data_path_for_version() is data_path.
        check_version(
            &config.data_path_for_version(),
            &self.kernel_version,
            latest_compatible_version(),
        )
    }
}

/// # Panics
///
/// If p.parent() is None.
fn check_version(p: &Path, current: &Version, least_compatible: Version) -> Result<()> {
    let ver_str = match std::fs::read_to_string(p) {
        Ok(x) => x,
        Err(e) if e.kind() == io::ErrorKind::NotFound => "".into(),
        Err(e) => return Err(Error::ReadingVersion(e)),
    };

    if ver_str.is_empty() {
        atomic_write(p, current.to_string().as_bytes()).map_err(Error::WritingVersion)?;
        return Ok(());
    }

    let prev_version = Version::new(&ver_str);
    if prev_version < least_compatible {
        return Err(Error::CheckingVersion(Box::new(CheckingVersionError {
            least_compatible,
            data: prev_version,
            current: current.clone(),
        })));
    }
    atomic_write(p, current.to_string().as_bytes()).map_err(Error::WritingVersion)?;
    Ok(())
}

/// Write content to p atomically. Create the parent directory if it doesn't
/// already exist.
///
/// # Panics
///
/// if p.parent() is None.
fn atomic_write(p: &Path, content: &[u8]) -> io::Result<()> {
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
    Version::new("0.1.0-beta.1")
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
    fn test_check_version() -> Result<()> {
        let tmp = NamedTempFile::new().unwrap();
        let p = tmp.path();
        // We just want NamedTempFile to delete the file on drop. We want to
        // start with the file not exist.
        std::fs::remove_file(p).unwrap();

        let latest_compatible: Version = "0.1.0-alpha.9".parse().unwrap();

        check_version(p, &"0.1.15".parse().unwrap(), latest_compatible.clone())?;
        assert_eq!(std::fs::read_to_string(p).unwrap(), "0.1.15");

        check_version(p, &"0.2.0".parse().unwrap(), latest_compatible)?;
        assert_eq!(std::fs::read_to_string(p).unwrap(), "0.2.0");

        Ok(())
    }

    #[test]
    fn test_check_version_failure() {
        let tmp = NamedTempFile::new().unwrap();
        let p = tmp.path();
        check_version(p, &"0.1.0".parse().unwrap(), "0.1.0".parse().unwrap()).unwrap();
        let err =
            check_version(p, &"0.2.2".parse().unwrap(), "0.2.0".parse().unwrap()).unwrap_err();
        match err {
            Error::CheckingVersion(e) => assert_eq!(*e, CheckingVersionError {
                current:          "0.2.2".parse().unwrap(),
                least_compatible: "0.2.0".parse().unwrap(),
                data:             "0.1.0+unknown".parse().unwrap(),
            }),
            e => panic!("unexpected error {e}"),
        }
    }
}
