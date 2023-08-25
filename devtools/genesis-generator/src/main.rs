#![allow(clippy::uninlined_format_args)]

use std::fs;

use clap::{load_yaml, value_t, App};
use ethers_core::abi::AbiEncode;

use common_config_parser::{parse_file, types::Config};
use common_crypto::{
    BlsPrivateKey, BlsPublicKey, PrivateKey, PublicKey, Secp256k1PrivateKey,
    Secp256k1RecoverablePrivateKey, Signature, ToBlsPublicKey, ToPublicKey, UncompressedPublicKey,
};
use core_executor::system_contract::metadata::metadata_abi;
use protocol::codec::hex_decode;
use protocol::types::{
    Address, Hasher, Hex, Metadata, RichBlock, SignedTransaction, UnsignedTransaction,
    UnverifiedTransaction, ValidatorExtend, H160,
};

fn genesis_generator(
    priv_key: Secp256k1RecoverablePrivateKey,
    chain_id: u64,
    metadata: (Metadata, Metadata),
) {
    // template json file path
    let input_path = "../chain/genesis_single_node.json";
    // read block info from template json file
    let mut genesis: RichBlock =
        serde_json::from_slice(&std::fs::read(input_path).unwrap()).unwrap();

    let data_0 =
        metadata_abi::MetadataContractCalls::AppendMetadata(metadata_abi::AppendMetadataCall {
            metadata: metadata.0.into(),
        })
        .encode();
    let data_1 =
        metadata_abi::MetadataContractCalls::AppendMetadata(metadata_abi::AppendMetadataCall {
            metadata: metadata.1.into(),
        })
        .encode();

    for (idx, tx) in genesis.txs.iter_mut().enumerate() {
        let mut utx = tx.transaction.unsigned.clone();

        if idx == 1 {
            utx.set_data(data_0.clone().into());
        } else if idx == 2 {
            utx.set_data(data_1.clone().into())
        }

        let new_tx = build_axon_txs(&priv_key, utx, chain_id);
        *tx = new_tx;
    }

    // get new tx_hashes and update old
    let hashes = genesis
        .txs
        .iter()
        .map(|tx| tx.transaction.hash)
        .collect::<Vec<_>>();
    genesis.block.tx_hashes = hashes;

    // update chain_id in block header
    genesis.block.header.chain_id = chain_id;

    let output_genesis_str = serde_json::to_string_pretty(&genesis).unwrap();
    let path = "./temp";
    let _ = std::fs::create_dir_all(path);
    fs::write("./temp/new-genesis.json", output_genesis_str).unwrap();
}

// build txs
fn build_axon_txs(
    priv_key: &Secp256k1RecoverablePrivateKey,
    tx: UnsignedTransaction,
    id: u64,
) -> SignedTransaction {
    let signature = priv_key.sign_message(
        &Hasher::digest(tx.encode(Some(id), None))
            .as_bytes()
            .try_into()
            .unwrap(),
    );
    let utx = UnverifiedTransaction {
        unsigned:  tx,
        signature: Some(signature.to_bytes().into()),
        chain_id:  Some(id),
        hash:      Default::default(),
    }
    .calc_hash();

    SignedTransaction::from_unverified(utx, None).unwrap()
}

// get metadata key from config.toml
fn get_metadata(config_path: String) -> (Metadata, Metadata) {
    let entries = fs::read_dir(config_path)
        .unwrap()
        .filter_map(|res| {
            res.ok().and_then(|e| {
                let path = e.path();
                if path.extension().map(|e| e == "toml").unwrap_or_default() {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();
    let input_path = "metadata.json";
    let mut metadata: Metadata =
        serde_json::from_slice(&std::fs::read(input_path).unwrap()).unwrap();

    // get propose_weight and vote_weight from metadata template
    let propose_weight = metadata.verifier_list[0].propose_weight;
    let vote_weight = metadata.verifier_list[0].vote_weight;

    // generate Vec<ValidatorExtend>
    let ve_list = entries
        .iter()
        .map(|file_name| {
            let config: Config = parse_file(file_name, false).unwrap();
            let priv_key = config.privkey;
            get_ve(Hex::encode(priv_key.as_ref()), propose_weight, vote_weight)
        })
        .collect::<Vec<_>>();

    metadata.verifier_list = ve_list;

    let mut metadata_1 = metadata.clone();
    metadata_1.epoch = metadata.epoch + 1;
    metadata_1.version.start = metadata.version.end + 1;
    metadata_1.version.end = metadata_1.version.start + metadata.version.end - 1;

    (metadata, metadata_1)
}

// get ValidatorExtend properties values
fn get_ve(priv_key: Hex, propose_weight: u32, vote_weight: u32) -> ValidatorExtend {
    let hex_privkey = hex_decode(&priv_key.as_string_trim0x()).unwrap();
    let my_privkey = Secp256k1PrivateKey::try_from(hex_privkey.as_slice()).unwrap();
    let my_pubkey = my_privkey.pub_key();
    let my_address = Address::from_pubkey_bytes(my_pubkey.to_uncompressed_bytes()).unwrap();
    let bls_priv_key = BlsPrivateKey::try_from(hex_privkey.as_ref()).unwrap();
    let bls_public_key: BlsPublicKey = bls_priv_key.pub_key(&"".to_string());

    ValidatorExtend {
        bls_pub_key: Hex::encode(&bls_public_key.to_bytes()),
        pub_key: Hex::encode(&my_pubkey.to_bytes()),
        address: H160::from_slice(my_address.as_slice()),
        propose_weight,
        vote_weight,
    }
}

fn main() {
    let yml = load_yaml!("config.yml");
    let m = App::from_yaml(yml).get_matches();
    let private_key = value_t!(m, "private_key", String).unwrap();
    // private key
    let priv_key =
        Secp256k1RecoverablePrivateKey::try_from(hex_decode(&private_key).unwrap().as_slice())
            .expect("Invalid secp private key");
    // chain_id
    let chain_id = value_t!(m, "chain_id", u64).unwrap_or_default();

    // config path
    let config_path = value_t!(m, "config_path", String).unwrap_or_default();
    let me = get_metadata(config_path);
    genesis_generator(priv_key, chain_id, me);
}
