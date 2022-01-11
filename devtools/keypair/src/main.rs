#[macro_use]
extern crate clap;

use std::convert::TryFrom;
use std::default::Default;

use clap::App;
use ophelia::{PrivateKey, PublicKey, ToBlsPublicKey};
use ophelia_blst::BlsPrivateKey;
use protocol::types::{Address, Bytes};
use rand::rngs::OsRng;
use serde::Serialize;
use tentacle_secio::SecioKeyPair;

#[derive(Default, Serialize, Debug)]
struct Keypair {
    pub index:          usize,
    pub private_key:    String,
    pub public_key:     String,
    pub address:        String,
    pub peer_id:        String,
    pub bls_public_key: String,
}

#[derive(Default, Serialize, Debug)]
struct Output {
    pub common_ref: String,
    pub keypairs:   Vec<Keypair>,
}

#[allow(clippy::needless_range_loop)]
pub fn main() {
    let yml = load_yaml!("keypair.yml");
    let m = App::from(yml).get_matches();
    let number = value_t!(m, "number", usize).unwrap();
    let priv_keys = values_t!(m.values_of("private_keys"), String).unwrap_or_default();
    let len = priv_keys.len();
    if len > number {
        panic!("private keys length can not be larger than number");
    }

    let mut output = Output {
        common_ref: add_0x(String::from("0")),
        keypairs:   vec![],
    };

    for i in 0..number {
        let mut k = Keypair::default();
        let seckey = if i < len {
            Bytes::from(hex::decode(&priv_keys[i]).expect("decode hex private key"))
        } else {
            BlsPrivateKey::generate(&mut OsRng).to_bytes()
        };

        let keypair = SecioKeyPair::secp256k1_raw_key(seckey.as_ref()).expect("secp256k1 keypair");
        let pubkey = keypair.public_key().inner();
        let address = Address::from_pubkey_bytes(pubkey.clone()).unwrap();

        k.private_key = add_0x(hex::encode(seckey.as_ref()));
        k.public_key = add_0x(hex::encode(pubkey));
        k.peer_id = keypair.public_key().peer_id().to_base58();
        k.address = add_0x(hex::encode(address.as_slice()));

        let priv_key = BlsPrivateKey::try_from(seckey.as_ref()).unwrap();
        let pub_key = priv_key.pub_key(&output.common_ref);
        k.bls_public_key = add_0x(hex::encode(pub_key.to_bytes()));
        k.index = i + 1;
        output.keypairs.push(k);
    }
    let output_str = serde_json::to_string_pretty(&output).unwrap();
    println!("{}", output_str);
}

fn add_0x(s: String) -> String {
    "0x".to_owned() + &s
}
