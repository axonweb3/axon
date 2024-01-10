use std::path::{Path, PathBuf};
use std::{fs::File, io::Write};

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
        let path = Path::new(&path);

        for i in 0..num {
            let key_pair = Keypair::generate(i)?;
            write_private_keys(
                path,
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
    pub peer_id:         String,
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
            peer_id:         secio_keypair.public_key().peer_id().to_base58(),
            bls_private_key: Hex::encode(&bls_seckey),
            bls_public_key:  Hex::encode(bls_pub_key.to_bytes()),
        })
    }

    #[cfg(test)]
    pub(crate) fn check(&self) {
        use common_crypto::{BlsSignatureVerify, HashValue, ToPublicKey};
        use tentacle_secio::KeyProvider;

        let another_pubkey = Secp256k1PrivateKey::try_from(self.net_private_key.as_ref())
            .unwrap()
            .pub_key();
        assert_eq!(self.public_key, Hex::encode(another_pubkey.to_bytes()));
        assert_ne!(self.net_private_key, self.bls_private_key);

        let msg = HashValue::from_bytes_unchecked(protocol::types::Hasher::digest("axon").0);
        let net_priv_key = SecioKeyPair::secp256k1_raw_key(self.net_private_key.as_ref()).unwrap();
        let bls_priv_key = BlsPrivateKey::try_from(self.bls_private_key.as_ref()).unwrap();
        let net_sig = net_priv_key.sign_ecdsa(&msg).unwrap();
        let bls_sig = bls_priv_key.sign_message(&msg);
        let net_pub_key = net_priv_key.public_key();
        let bls_pub_key = bls_priv_key.pub_key(&String::new());

        assert!(bls_sig.verify(&msg, &bls_pub_key, &String::new()).is_ok());
        assert!(net_priv_key.verify_ecdsa(net_pub_key.inner_ref(), &msg, net_sig));
    }
}

#[derive(Serialize, Clone, Debug)]
struct Output {
    keypairs: Vec<Keypair>,
}

fn write_private_keys(path: &Path, net_key: Bytes, bls_key: Bytes, index: usize) -> Result<()> {
    let write = |path: PathBuf, data: Bytes| -> Result<()> {
        let mut file = File::create(path).map_err(Error::WritingPrivateKey)?;
        file.write_all(&data).map_err(Error::WritingPrivateKey)?;

        Ok(())
    };

    let mut bls_key_path = path.to_path_buf();
    bls_key_path.push(format!("bls_{}.key", index));
    let mut net_key_path = path.to_path_buf();
    net_key_path.push(format!("net_{}.key", index));

    write(bls_key_path, bls_key)?;
    write(net_key_path, net_key)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = Keypair::generate(1).unwrap();
        keypair.check();
    }
}
