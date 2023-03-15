use std::error::Error;

use clap::Parser;
use ethers_contract::Abigen;

#[derive(Clone, Debug, Parser)]
struct Args {
    #[arg(short = 'c', long)]
    contract_name: String,

    #[arg(short = 'j', long)]
    json_abi_path: String,

    #[arg(short = 'o', long)]
    output_file_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    Abigen::new(args.contract_name, args.json_abi_path)?
        .generate()?
        .write_to_file(args.output_file_path)?;
    Ok(())
}
