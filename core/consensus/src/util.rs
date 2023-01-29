use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use overlord::Crypto;
use parking_lot::RwLock;

use crate::ConsensusError;
use common_crypto::{
    BlsPrivateKey, BlsPublicKey, BlsSignature, BlsSignatureVerify, HashValue, PrivateKey, Signature,
};
use protocol::traits::Context;
use protocol::types::{Address, Bytes, Hash, Hasher, Hex, MerkleRoot, SignedTransaction};
use protocol::{ProtocolError, ProtocolResult};

pub fn digest_signed_transactions(stxs: &[SignedTransaction]) -> Hash {
    Hasher::digest(rlp::encode_list(stxs))
}

pub fn time_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct OverlordCrypto {
    private_key: BlsPrivateKey,
    addr_pubkey: RwLock<HashMap<Bytes, BlsPublicKey>>,
    common_ref:  String,
}

impl Crypto for OverlordCrypto {
    fn hash(&self, msg: Bytes) -> Bytes {
        Bytes::from(Hasher::digest(msg).as_bytes().to_vec())
    }

    fn sign(&self, hash: Bytes) -> Result<Bytes, Box<dyn Error + Send>> {
        let hash = HashValue::try_from(hash.as_ref()).map_err(|_| {
            ProtocolError::from(ConsensusError::Other(
                "failed to convert hash value".to_string(),
            ))
        })?;
        let sig = self.private_key.sign_message(&hash);
        Ok(sig.to_bytes())
    }

    fn verify_signature(
        &self,
        signature: Bytes,
        hash: Bytes,
        voter: Bytes,
    ) -> Result<(), Box<dyn Error + Send>> {
        let map = self.addr_pubkey.read();
        let hash = HashValue::try_from(hash.as_ref()).map_err(|_| {
            ProtocolError::from(ConsensusError::Other(
                "failed to convert hash value".to_string(),
            ))
        })?;
        let pub_key = map.get(&voter).ok_or_else(|| {
            ProtocolError::from(ConsensusError::Other("lose public key".to_string()))
        })?;
        let signature = BlsSignature::try_from(signature.as_ref())
            .map_err(|e| ProtocolError::from(ConsensusError::CryptoErr(Box::new(e))))?;

        signature
            .verify(&hash, pub_key, &self.common_ref)
            .map_err(|e| ProtocolError::from(ConsensusError::CryptoErr(Box::new(e))))?;
        Ok(())
    }

    fn aggregate_signatures(
        &self,
        signatures: Vec<Bytes>,
        voters: Vec<Bytes>,
    ) -> Result<Bytes, Box<dyn Error + Send>> {
        if signatures.len() != voters.len() {
            return Err(ProtocolError::from(ConsensusError::Other(
                "signatures length does not match voters length".to_string(),
            ))
            .into());
        }

        let map = self.addr_pubkey.read();
        let mut sigs_pubkeys = Vec::with_capacity(signatures.len());
        for (sig, addr) in signatures.iter().zip(voters.iter()) {
            let signature = BlsSignature::try_from(sig.as_ref())
                .map_err(|e| ProtocolError::from(ConsensusError::CryptoErr(Box::new(e))))?;

            let pub_key = map.get(addr).ok_or_else(|| {
                ProtocolError::from(ConsensusError::Other("lose public key".to_string()))
            })?;

            sigs_pubkeys.push((signature, pub_key.to_owned()));
        }

        let sig = BlsSignature::combine(sigs_pubkeys)
            .map_err(|e| ProtocolError::from(ConsensusError::CryptoErr(Box::new(e.into()))))?;
        Ok(sig.to_bytes())
    }

    fn verify_aggregated_signature(
        &self,
        aggregated_signature: Bytes,
        hash: Bytes,
        voters: Vec<Bytes>,
    ) -> Result<(), Box<dyn Error + Send>> {
        let map = self.addr_pubkey.read();
        let mut pub_keys = Vec::with_capacity(voters.len());

        for addr in voters.iter() {
            let pub_key = map.get(addr).ok_or_else(|| {
                ProtocolError::from(ConsensusError::Other("lose public key".to_string()))
            })?;
            pub_keys.push(pub_key.clone());
        }

        self.inner_verify_aggregated_signature(hash, pub_keys, aggregated_signature)?;
        Ok(())
    }
}

impl OverlordCrypto {
    pub fn new(
        private_key: BlsPrivateKey,
        pubkey_to_bls_pubkey: HashMap<Bytes, BlsPublicKey>,
        common_ref: String,
    ) -> Self {
        OverlordCrypto {
            addr_pubkey: RwLock::new(pubkey_to_bls_pubkey),
            private_key,
            common_ref,
        }
    }

    pub fn update(&self, new_addr_pubkey: HashMap<Bytes, BlsPublicKey>) {
        let mut map = self.addr_pubkey.write();

        *map = new_addr_pubkey;
    }

    pub fn inner_verify_aggregated_signature(
        &self,
        hash: Bytes,
        pub_keys: Vec<BlsPublicKey>,
        signature: Bytes,
    ) -> ProtocolResult<()> {
        let aggregate_key = BlsPublicKey::aggregate(pub_keys)
            .map_err(|e| ConsensusError::CryptoErr(Box::new(e.into())))?;
        let aggregated_signature = BlsSignature::try_from(signature.as_ref())
            .map_err(|e| ConsensusError::CryptoErr(Box::new(e)))?;
        let hash = HashValue::try_from(hash.as_ref())
            .map_err(|_| ConsensusError::Other("failed to convert hash value".to_string()))?;

        aggregated_signature
            .verify(&hash, &aggregate_key, &self.common_ref)
            .map_err(|e| ConsensusError::CryptoErr(Box::new(e)))?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ExecuteInfo {
    pub ctx:          Context,
    pub height:       u64,
    pub chain_id:     Hash,
    pub block_hash:   Hash,
    pub signed_txs:   Vec<SignedTransaction>,
    pub order_root:   MerkleRoot,
    pub cycles_price: u64,
    pub proposer:     Address,
    pub timestamp:    u64,
    pub cycles_limit: u64,
}

pub fn convert_hex_to_bls_pubkeys(hex: Hex) -> ProtocolResult<BlsPublicKey> {
    let hex_pubkey = hex.as_bytes();
    let ret = BlsPublicKey::try_from(hex_pubkey.as_ref())
        .map_err(|e| ConsensusError::CryptoErr(Box::new(e)))?;
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use protocol::codec::hex_decode;

    #[test]
    fn test_blst() {
        let private_keys = vec![
            hex_decode("37aa0f893d05914a4def0460c0a984d3611546cfb26924d7a7ca6e0db9950a2d").unwrap(),
            hex_decode("383fcff8683b8115e31613949be24254b4204ffbe43c227408a76334a2e3fb32").unwrap(),
            hex_decode("51ce21643b911347c5d5c85c323d9d5421810dc89f46b688720b2715f5e8e936").unwrap(),
            hex_decode("69ff51f4c22f30615f68b88efa740f8f1b9169e88842b83d189748d06f1a948e").unwrap(),
        ];

        let public_keys = vec![
            hex_decode("ac85bbb40347b6e06ac2dc2da1f75eece029cdc0ed2d456c457d27e288bfbfbcd4c5c19716e9b250134a0e76ce50fa22").unwrap(),
            hex_decode("91ed9f3c51c580e56948b1bda9d00c2159665f8a6e284191ab816ee64ef2487d78453a547a0f14efbf842bba5b5a3b4f").unwrap(),
            hex_decode("92e5d0856fb20ea9cb5ab5da2d3331c38d32cc96507f6ad902fa3da9400096a485fb4e09834bc93de55db224f26c229c").unwrap(),
            hex_decode("a694f4e48a5a173b61731998f8f1204342dc5c8eb1e32cdae37415c20d11ae035ddac4a39f105e9c2d4d3691024d385d").unwrap(),
        ];

        let msg = Hasher::digest(Bytes::from("axon-consensus"));
        let hash = HashValue::try_from(msg.as_bytes()).unwrap();
        let mut sigs_and_pub_keys = Vec::new();
        for i in 0..3 {
            let sig = BlsPrivateKey::try_from(private_keys[i].as_ref())
                .unwrap()
                .sign_message(&hash);
            let pub_key = BlsPublicKey::try_from(public_keys[i].as_ref()).unwrap();
            sigs_and_pub_keys.push((sig, pub_key));
        }

        let signature = BlsSignature::combine(sigs_and_pub_keys.clone()).unwrap();
        let aggregate_key = BlsPublicKey::aggregate(
            sigs_and_pub_keys
                .iter()
                .map(|s| s.1.clone())
                .collect::<Vec<_>>(),
        )
        .unwrap();

        let res = signature.verify(&hash, &aggregate_key, &"axon".into());
        assert!(res.is_ok());
    }

    #[test]
    fn test_aggregate_pubkeys_order() {
        let public_keys = vec![
            hex_decode("ac85bbb40347b6e06ac2dc2da1f75eece029cdc0ed2d456c457d27e288bfbfbcd4c5c19716e9b250134a0e76ce50fa22").unwrap(),
            hex_decode("91ed9f3c51c580e56948b1bda9d00c2159665f8a6e284191ab816ee64ef2487d78453a547a0f14efbf842bba5b5a3b4f").unwrap(),
            hex_decode("92e5d0856fb20ea9cb5ab5da2d3331c38d32cc96507f6ad902fa3da9400096a485fb4e09834bc93de55db224f26c229c").unwrap(),
            hex_decode("a694f4e48a5a173b61731998f8f1204342dc5c8eb1e32cdae37415c20d11ae035ddac4a39f105e9c2d4d3691024d385d").unwrap(),
        ];
        let mut pub_keys = public_keys
            .into_iter()
            .map(|pk| BlsPublicKey::try_from(pk.as_ref()).unwrap())
            .collect::<Vec<_>>();
        let pk_1 = BlsPublicKey::aggregate(pub_keys.clone()).unwrap();
        pub_keys.reverse();
        let pk_2 = BlsPublicKey::aggregate(pub_keys).unwrap();
        assert_eq!(pk_1, pk_2);
    }

    #[test]
    fn test_convert_from_hex() {
        let hex_str = "0xa694f4e48a5a173b61731998f8f1204342dc5c8eb1e32cdae37415c20d11ae035ddac4a39f105e9c2d4d3691024d385d";
        assert!(
            convert_hex_to_bls_pubkeys(Hex::from_string(String::from(hex_str)).unwrap()).is_ok()
        );
    }
}
