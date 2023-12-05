#[macro_use]
extern crate clap;

use std::{convert::TryFrom, fs, path::PathBuf};

use clap::App;
use common_crypto::{BlsPrivateKey, PrivateKey, PublicKey, Secp256k1PrivateKey, ToBlsPublicKey};
use protocol::{codec::hex_encode, types::Address};
use rand::rngs::OsRng;
use serde::Serialize;
use tentacle_secio::SecioKeyPair;

#[derive(Default, Serialize, Debug)]
struct Keypair {
    pub index:           usize,
    pub net_private_key: String,
    pub public_key:      String,
    pub address:         String,
    pub peer_id:         String,
    pub bls_private_key: String,
    pub bls_public_key:  String,
}

#[derive(Default, Serialize, Debug)]
struct Output {
    pub common_ref: String,
    pub keypairs:   Vec<Keypair>,
}

#[allow(clippy::needless_range_loop, clippy::uninlined_format_args)]
pub fn main() {
    let yml = load_yaml!("keypair.yml");
    let m = App::from(yml).get_matches();
    let number = value_t!(m, "number", usize).unwrap();
    let path = value_t!(m, "binary-path", String).unwrap();
    let path = PathBuf::from(path);
    let _ = fs::create_dir(path.clone());

    let mut output = Output {
        common_ref: add_0x(String::from("0")),
        keypairs:   vec![],
    };

    for i in 0..number {
        let mut k = Keypair::default();
        let bls_seckey = BlsPrivateKey::generate(&mut OsRng).to_bytes();
        let net_seckey = Secp256k1PrivateKey::generate(&mut OsRng).to_bytes();

        let keypair = SecioKeyPair::secp256k1_raw_key(&net_seckey).unwrap();
        let pubkey = keypair.public_key().inner();
        let address = Address::from_pubkey_bytes(pubkey.clone()).unwrap();

        k.net_private_key = add_0x(hex_encode(net_seckey.as_ref()));
        k.public_key = add_0x(hex_encode(pubkey));
        k.peer_id = keypair.public_key().peer_id().to_base58();
        k.address = add_0x(hex_encode(address.as_slice()));

        let bls_priv_key = BlsPrivateKey::try_from(bls_seckey.as_ref()).unwrap();
        let bls_pub_key = bls_priv_key.pub_key(&output.common_ref);
        k.bls_private_key = add_0x(hex_encode(bls_seckey.as_ref()));
        k.bls_public_key = add_0x(hex_encode(bls_pub_key.to_bytes()));
        k.index = i;
        output.keypairs.push(k);

        write_private_key(path.clone(), bls_seckey.to_vec(), true, i);
        write_private_key(path.clone(), net_seckey.to_vec(), false, i);
    }
    let output_str = serde_json::to_string_pretty(&output).unwrap();
    println!("{}", output_str);
}

fn add_0x(s: String) -> String {
    "0x".to_owned() + &s
}

fn write_private_key(mut path: PathBuf, key: Vec<u8>, is_bls: bool, index: usize) {
    use std::fs::File;
    use std::io::Write;

    if is_bls {
        let file_name = format!("bls_{}.key", index);
        path.push(file_name);
    } else {
        let file_name = format!("net_{}.key", index);
        path.push(file_name);
    }

    let mut file = File::create(path).unwrap();
    file.write_all(&key).unwrap();
}
