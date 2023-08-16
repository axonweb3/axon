use std::{
    io::{self, Write},
    path::Path,
};

use common_version::Version;

use common_config_parser::types::Config;

use crate::{CheckingVersionError, Error, Result};

/// # Panics
///
/// If p.parent() is None.
pub(crate) fn check_version(p: &Path, current: &Version, least_compatible: Version) -> Result<()> {
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

pub(crate) fn latest_compatible_version() -> Version {
    Version::new("0.1.0-beta.1")
}

pub(crate) fn register_log(config: &Config) {
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
