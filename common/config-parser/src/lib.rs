pub mod types;
use serde::de;

use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

/// Parse a config from reader.
pub fn parse_reader<R: io::Read, T: de::DeserializeOwned>(r: &mut R) -> Result<T, ParseError> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)?;
    Ok(toml::from_slice(&buf)?)
}

pub fn parse_json<R: io::Read, T: de::DeserializeOwned>(r: &mut R) -> Result<T, ParseError> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)?;
    Ok(serde_json::from_slice(&buf)?)
}

/// Parse a config from file.
///
/// Note: In most cases, function `parse` is better.
pub fn parse_file<T: de::DeserializeOwned>(
    name: impl AsRef<Path>,
    is_json: bool,
) -> Result<T, ParseError> {
    let mut f = fs::File::open(name)?;
    if is_json {
        parse_json(&mut f)
    } else {
        parse_reader(&mut f)
    }
}

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    Deserialize(toml::de::Error),
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
}

impl error::Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::IO(e) => write!(f, "{}", e),
            ParseError::Deserialize(e) => write!(f, "{}", e),
            ParseError::Reqwest(e) => write!(f, "{}", e),
            ParseError::Json(e) => write!(f, "{}", e),
        }
    }
}

impl From<io::Error> for ParseError {
    fn from(error: io::Error) -> ParseError {
        ParseError::IO(error)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(error: serde_json::Error) -> ParseError {
        ParseError::Json(error)
    }
}

impl From<toml::de::Error> for ParseError {
    fn from(error: toml::de::Error) -> ParseError {
        ParseError::Deserialize(error)
    }
}

impl From<reqwest::Error> for ParseError {
    fn from(error: reqwest::Error) -> ParseError {
        ParseError::Reqwest(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_file, parse_reader};
    use serde::Deserialize;
    use stringreader::StringReader;

    #[derive(Debug, Deserialize)]
    struct Config {
        global_string: Option<String>,
        global_int:    Option<u64>,
    }

    #[test]
    fn test_parse_config() {
        let file_path = "../../devtools/chain/config.toml";
        let _config: Config = parse_file(&file_path, false).unwrap();
    }

    #[test]
    fn test_parse_reader() {
        let toml_str = r#"
        global_string = "Best Food"
        global_int = 42
    "#;
        let mut toml_r = StringReader::new(toml_str);
        let config: Config = parse_reader(&mut toml_r).unwrap();
        assert_eq!(config.global_string, Some(String::from("Best Food")));
        assert_eq!(config.global_int, Some(42));
    }

    #[ignore]
    #[test]
    fn test_parse_file() {
        let config: Config = parse_file("/tmp/config.toml", false).unwrap();
        assert_eq!(config.global_string, Some(String::from("Best Food")));
        assert_eq!(config.global_int, Some(42));
    }
}
