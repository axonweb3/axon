use std::{fs::File, io::Write, path::PathBuf};

use clap::Parser;
use serde::Serialize;

use common_crypto::{BlsPrivateKey, PrivateKey, PublicKey, Secp256k1PrivateKey, ToBlsPublicKey};
use protocol::rand::rngs::OsRng;
use protocol::types::{Address, Bytes, Hex};
use tentacle_secio::SecioKeyPair;

use crate::error::{Error, Result};

#[derive(Parser, Debug)]
#[command(about = "Initialize new axon data directory")]
pub struct GenerateKeypairArgs {
    #[arg(
        short = 'n',
        long = "number",
        value_name = "NUMBER",
        help = "The number of keypairs to generate.",
        default_value = "1"
    )]
    pub num:  usize,
    #[arg(
        short = 'p',
        long = "path",
        value_name = "PRIVATE_KEY_PATH",
        help = "The path to store the generated private key binary.",
        default_value = "free-space"
    )]
    pub path: String,
}

impl GenerateKeypairArgs {
    pub(crate) fn execute(self) -> Result<()> {
        let Self { num, path } = self;
        let mut keypairs = Vec::with_capacity(num);
        let path = PathBuf::from(path);

        for i in 0..num {
            let key_pair = Keypair::generate(i)?;
            write_private_keys(
                &path,
                key_pair.net_private_key.as_bytes(),
                key_pair.bls_private_key.as_bytes(),
                i,
            )?;
            keypairs.push(Keypair::generate(i)?);
        }

        println!(
            "{}",
            serde_json::to_string_pretty(&Output { keypairs }).unwrap()
        );

        Ok(())
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct Keypair {
    pub index:           usize,
    pub net_private_key: Hex,
    pub public_key:      Hex,
    pub address:         Address,
    pub peer_id:         Hex,
    pub bls_private_key: Hex,
    pub bls_public_key:  Hex,
}

impl Keypair {
    pub(crate) fn generate(i: usize) -> Result<Self> {
        let bls_seckey = BlsPrivateKey::generate(&mut OsRng).to_bytes();
        let net_seckey = Secp256k1PrivateKey::generate(&mut OsRng).to_bytes();
        Self::from_private_keys(net_seckey, bls_seckey, i)
    }

    pub(crate) fn from_private_keys(
        net_seckey: Bytes,
        bls_seckey: Bytes,
        i: usize,
    ) -> Result<Keypair> {
        let secio_keypair = SecioKeyPair::secp256k1_raw_key(&net_seckey)
            .map_err(|e| Error::Crypto(e.to_string()))?;
        let pubkey = secio_keypair.public_key().inner();

        let bls_priv_key = BlsPrivateKey::try_from(bls_seckey.as_ref())
            .map_err(|e| Error::Crypto(e.to_string()))?;
        let bls_pub_key = bls_priv_key.pub_key(&String::new());

        Ok(Keypair {
            index:           i,
            net_private_key: Hex::encode(&net_seckey),
            public_key:      Hex::encode(&pubkey),
            address:         Address::from_pubkey_bytes(pubkey).map_err(Error::Running)?,
            peer_id:         Hex::encode(secio_keypair.public_key().peer_id().to_base58()),
            bls_private_key: Hex::encode(bls_seckey.to_vec()),
            bls_public_key:  Hex::encode(bls_pub_key.to_bytes()),
        })
    }
}

#[derive(Serialize, Clone, Debug)]
struct Output {
    keypairs: Vec<Keypair>,
}

fn write_private_keys(path: &PathBuf, net_key: Bytes, bls_key: Bytes, index: usize) -> Result<()> {
    let write = |path: PathBuf, data: Bytes| -> Result<()> {
        let mut file = File::create(path).map_err(Error::WritingPrivateKey)?;
        file.write_all(&data).map_err(Error::WritingPrivateKey)?;

        Ok(())
    };

    let mut bls_key_path = path.clone();
    bls_key_path.push(format!("bls_{}.key", index));
    let mut net_key_path = path.clone();
    net_key_path.push(format!("net_{}.key", index));

    write(bls_key_path, bls_key)?;
    write(net_key_path, net_key)?;
    Ok(())
}
