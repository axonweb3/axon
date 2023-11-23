use std::{ffi::OsStr, marker::PhantomData, path::PathBuf};

use clap::builder::{StringValueParser, TypedValueParser};
use serde::de;

use crate::parse_file;

mod config;
pub mod spec;

pub use config::*;

#[derive(Clone, Debug)]
pub struct JsonValueParser<T: de::DeserializeOwned + 'static + Clone + Send + Sync>(PhantomData<T>);

impl<T> Default for JsonValueParser<T>
where
    T: de::DeserializeOwned + 'static + Clone + Send + Sync,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> TypedValueParser for JsonValueParser<T>
where
    T: de::DeserializeOwned + 'static + Clone + Send + Sync,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let file_path = StringValueParser::new()
            .parse_ref(cmd, arg, value)
            .map(PathBuf::from)?;
        parse_file(&file_path, true).map_err(|err| {
            let kind = clap::error::ErrorKind::InvalidValue;
            let msg =
                format!(
                    "failed to parse JSON file {} since {err}",
                    file_path.display()
                );
            clap::Error::raw(kind, msg)
        })
    }
}
