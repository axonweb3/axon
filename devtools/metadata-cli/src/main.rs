use std::path::PathBuf;

use anyhow::{Context, Result};
use axon_types::{
    basic::{Byte33, Byte48, Identity, Uint32, Uint64},
    metadata::{Metadata, MetadataCellData, MetadataCellDataReader, MetadataList, ValidatorList},
};
use clap::{Parser, Subcommand};
use molecule::prelude::{Builder, Entity, Reader};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, PickFirst};

mod serde_helpers;
use serde_helpers::{HexBytes, HexU32, HexU64};

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct Validator {
    #[serde_as(as = "HexBytes")]
    pub bls_pub_key: [u8; 48],

    #[serde_as(as = "HexBytes")]
    pub pub_key: [u8; 33],

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
        let address =
            axon_protocol::types::Address::from_pubkey_bytes(value.pub_key.as_slice()).unwrap();
        Self::new_builder()
            .bls_pub_key(Byte48::from_slice(value.bls_pub_key.as_slice()).unwrap())
            .pub_key(Byte33::from_slice(value.pub_key.as_slice()).unwrap())
            .address(Identity::from_slice(address.as_slice()).unwrap())
            .propose_count(Uint64::from_slice(&value.propose_count.to_le_bytes()).unwrap())
            .propose_weight(Uint32::from_slice(&value.propose_weight.to_le_bytes()).unwrap())
            .vote_weight(Uint32::from_slice(&value.vote_weight.to_le_bytes()).unwrap())
            .build()
    }
}

impl From<&axon_types::metadata::ValidatorReader<'_>> for Validator {
    fn from(value: &axon_types::metadata::ValidatorReader) -> Self {
        Self {
            bls_pub_key:    value.bls_pub_key().as_slice().try_into().unwrap(),
            propose_count:  u64::from_le_bytes(
                value.propose_count().as_slice().try_into().unwrap(),
            ),
            propose_weight: u32::from_le_bytes(
                value.propose_weight().as_slice().try_into().unwrap(),
            ),
            pub_key:        value.pub_key().as_slice().try_into().unwrap(),
            vote_weight:    u32::from_le_bytes(value.vote_weight().as_slice().try_into().unwrap()),
        }
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
    ParseData(ParseData),
}

impl Command {
    fn run(self) -> Result<()> {
        match self {
            Command::GetData(g) => g.run(),
            Command::ParseData(p) => p.run(),
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

#[derive(Parser)]
struct ParseData {
    #[arg(short, long)]
    input:  PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>,
}

impl ParseData {
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

        let input = input.trim();
        let input = input.strip_prefix("0x").unwrap_or(input);
        let input = hex::decode(input).context("decoding input")?;

        let data = MetadataCellDataReader::from_slice(&input)
            .context("decoding input as MetadataCellData")?;

        if data.metadata().len() > 1 {
            eprintln!("Only showing the first metadata");
        }
        let metadata = data.metadata().get(0).context("no metadata")?;

        let result = Input {
            validators: metadata.validators().iter().map(|v| (&v).into()).collect(),
        };

        let result = toml::to_string_pretty(&result).context("serializing result")?;

        output
            .write_all(result.as_bytes())
            .context("writing output")?;

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

    use super::{GetData, ParseData};

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

    #[test]
    fn test_parse_data() -> Result<()> {
        ParseData::parse_from([
            "parse-data",
            "-i",
            concat!(env!("CARGO_MANIFEST_DIR"), "/data.example.hex"),
        ])
        .run()
    }
}
