use std::path::PathBuf;

use anyhow::{Context, Result};
use axon_types::{
    basic::{Byte33, Byte48, Identity, Uint32, Uint64},
    metadata::{Metadata, MetadataCellData, MetadataList, ValidatorList},
};
use clap::{Parser, Subcommand};
use molecule::prelude::{Builder, Entity};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, PickFirst};

mod serde_helpers;
use serde_helpers::{HexBytes, HexU32, HexU64};

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Validator {
    #[serde_as(as = "HexBytes")]
    pub bls_pub_key: [u8; 48],

    #[serde_as(as = "HexBytes")]
    pub pub_key: [u8; 33],

    #[serde_as(as = "HexBytes")]
    #[serde(default)]
    pub address: [u8; 20],

    #[serde_as(deserialize_as = "PickFirst<(_, HexU64)>")]
    #[serde(default)]
    pub propose_count: u64,

    #[serde_as(deserialize_as = "PickFirst<(_, HexU32)>")]
    #[serde(default)]
    pub propose_weight: u32,

    #[serde_as(deserialize_as = "PickFirst<(_, HexU32)>")]
    #[serde(default)]
    pub vote_weight: u32,
}

impl From<Validator> for axon_types::metadata::Validator {
    fn from(value: Validator) -> Self {
        Self::new_builder()
            .bls_pub_key(Byte48::from_slice(&value.bls_pub_key).unwrap())
            .pub_key(Byte33::from_slice(&value.pub_key).unwrap())
            .address(Identity::from_slice(&value.address).unwrap())
            .propose_count(Uint64::from_slice(&value.propose_count.to_le_bytes()).unwrap())
            .propose_weight(Uint32::from_slice(&value.propose_weight.to_le_bytes()).unwrap())
            .vote_weight(Uint32::from_slice(&value.vote_weight.to_le_bytes()).unwrap())
            .build()
    }
}

pub fn metadata_cell_data_with_validators(vs: ValidatorList) -> MetadataCellData {
    MetadataCellData::new_builder()
        .metadata(
            MetadataList::new_builder()
                .push(Metadata::new_builder().validators(vs).build())
                .build(),
        )
        .build()
}

#[derive(Deserialize, Serialize)]
pub struct Input {
    #[serde(alias = "verifier_list")]
    pub validators: Vec<Validator>,
}

#[derive(Deserialize, Serialize)]
pub struct Spec {
    params: Input,
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    GetData(GetData),
}

impl Command {
    fn run(self) -> Result<()> {
        match self {
            Command::GetData(g) => g.run(),
        }
    }
}

#[derive(Parser)]
struct GetData {
    #[arg(short, long)]
    input:  PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(long)]
    binary: bool,
}

impl GetData {
    fn run(self) -> Result<()> {
        let input = std::fs::read_to_string(self.input).context("read input file")?;
        let mut output: Box<dyn std::io::Write> = if let Some(o) = self.output {
            Box::new(
                std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(o)
                    .context("open output file")?,
            )
        } else {
            Box::new(std::io::stdout())
        };

        let input: Input = if input.trim_start().starts_with('{') {
            serde_json::from_str(&input).context("parsing input")?
        } else {
            let spec_or_input: toml::Value = toml::from_str(&input).context("parsing input")?;
            if spec_or_input.get("params").is_some_and(|v| v.is_table()) {
                toml::from_str::<Spec>(&input)
                    .context("parsing input")?
                    .params
            } else {
                toml::from_str::<Input>(&input).context("parsing input")?
            }
        };
        let mut vs = ValidatorList::new_builder();
        for v in input.validators {
            vs = vs.push(v.into());
        }
        let md = metadata_cell_data_with_validators(vs.build());
        if self.binary {
            output.write_all(md.as_slice()).context("writing output")?;
        } else {
            writeln!(output, "0x{}", hex::encode(md.as_slice())).context("writing output")?;
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    Cli::parse().command.run()
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use clap::Parser;

    use super::GetData;

    #[test]
    fn test_get_data() -> Result<()> {
        GetData::parse_from([
            "get-data",
            "-i",
            concat!(env!("CARGO_MANIFEST_DIR"), "/input.example.toml",),
        ])
        .run()
        .context("test toml")?;

        GetData::parse_from([
            "get-data",
            "-i",
            concat!(env!("CARGO_MANIFEST_DIR"), "/input.example.json",),
        ])
        .run()
        .context("test json")?;

        GetData::parse_from([
            "get-data",
            "-i",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../chain/specs/multi_nodes/chain-spec.toml",
            ),
        ])
        .run()
        .context("test spec")?;

        Ok(())
    }
}
