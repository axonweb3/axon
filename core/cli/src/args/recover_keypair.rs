use std::{fs::File, io::Read, path::PathBuf};

use clap::Parser;

use protocol::types::Bytes;

use crate::args::generate_keypair::Keypair;
use crate::error::{Error, Result};

#[derive(Parser, Debug)]
#[command(about = "Initialize new axon data directory")]
pub struct RecoverKeypairArgs {
    #[arg(
        short = 'n',
        long = "net_path",
        value_name = "NET_PRIVATE_KEY_PATH",
        help = "The path to store the net private key binary."
    )]
    pub net_private_key_path: String,
    #[arg(
        short = 'b',
        long = "bls_path",
        value_name = "BLS_PRIVATE_KEY_PATH",
        help = "The path to store the bls private key binary."
    )]
    pub bls_private_key_path: String,
}

impl RecoverKeypairArgs {
    pub(crate) fn execute(self) -> Result<()> {
        let Self {
            net_private_key_path,
            bls_private_key_path,
        } = self;
        let net_private_key = read_private_key(&PathBuf::from(net_private_key_path))?;
        let bls_private_key = read_private_key(&PathBuf::from(bls_private_key_path))?;

        let output = Keypair::from_private_keys(net_private_key, bls_private_key, 0)?;
        println!("{}", serde_json::to_string_pretty(&output).unwrap());

        Ok(())
    }
}

fn read_private_key(path: &PathBuf) -> Result<Bytes> {
    let mut file = File::open(path).map_err(Error::ReadingPrivateKey)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(Error::ReadingPrivateKey)?;
    Ok(buf.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_binary_priv_keys() {
        let net_key_path = "../../devtools/chain/net.key";
        let bls_key_path = "../../devtools/chain/bls.key";
        let net_private_key = read_private_key(&PathBuf::from(net_key_path)).unwrap();
        let bls_private_key = read_private_key(&PathBuf::from(bls_key_path)).unwrap();
        Keypair::from_private_keys(net_private_key, bls_private_key, 1)
            .unwrap()
            .check();
    }
}
