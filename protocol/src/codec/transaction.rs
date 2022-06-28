use bytes::BufMut;
use ethereum_types::BigEndianHash;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use common_crypto::secp256k1_recover;

use crate::lazy::CHAIN_ID;
use crate::types::{
    public_to_address, AccessList, AccessListItem, Bytes, BytesMut, Eip1559Transaction,
    Eip2930Transaction, Hasher, LegacyTransaction, Public, SignatureComponents, SignedTransaction,
    UnsignedTransaction, UnverifiedTransaction, H256, U256,
};

impl Encodable for SignatureComponents {
    fn rlp_append(&self, s: &mut RlpStream) {
        if self.is_eth_sig() {
            let r = U256::from(&self.r[0..32]);
            let s_ = U256::from(&self.s[0..32]);
            s.append(&self.standard_v).append(&r).append(&s_);
        } else {
            s.append(&self.standard_v).append(&self.r).append(&self.s);
        }
    }
}

impl SignatureComponents {
    fn rlp_decode(r: &Rlp, offset: usize, legacy_v: Option<u64>) -> Result<Self, DecoderError> {
        let v: u8 = if let Some(n) = legacy_v {
            SignatureComponents::extract_standard_v(n)
        } else {
            r.val_at(offset)?
        };

        let eth_tx_flag = v <= 1;

        Ok(SignatureComponents {
            standard_v: v,
            r:          if eth_tx_flag {
                let tmp: U256 = r.val_at(offset + 1)?;
                Bytes::from(<H256 as BigEndianHash>::from_uint(&tmp).as_bytes().to_vec())
            } else {
                r.val_at(offset + 1)?
            },
            s:          if eth_tx_flag {
                let tmp: U256 = r.val_at(offset + 2)?;
                Bytes::from(<H256 as BigEndianHash>::from_uint(&tmp).as_bytes().to_vec())
            } else {
                r.val_at(offset + 2)?
            },
        })
    }
}

impl LegacyTransaction {
    fn rlp_encode(
        &self,
        rlp: &mut RlpStream,
        chain_id: Option<u64>,
        signature: Option<&SignatureComponents>,
    ) {
        let rlp_stream_len = if signature.is_some() || chain_id.is_some() {
            9
        } else {
            6
        };

        rlp.begin_list(rlp_stream_len)
            .append(&self.nonce)
            .append(&self.gas_price)
            .append(&self.gas_limit)
            .append(&self.action)
            .append(&self.value)
            .append(&self.data);

        if let Some(sig) = signature {
            rlp.append(&sig.add_chain_replay_protection(chain_id))
                .append(&sig.r)
                .append(&sig.s);
        } else if let Some(id) = chain_id {
            rlp.append(&id).append(&0u8).append(&0u8);
        }
    }

    fn rlp_decode(r: &Rlp) -> Result<UnverifiedTransaction, DecoderError> {
        if r.item_count()? != 9 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        let tx = LegacyTransaction {
            nonce:     r.val_at(0)?,
            gas_price: r.val_at(1)?,
            gas_limit: r.val_at(2)?,
            action:    r.val_at(3)?,
            value:     r.val_at(4)?,
            data:      r.val_at(5)?,
        };

        let v: u64 = r.val_at(6)?;
        let id = SignatureComponents::extract_chain_id(v).unwrap_or_else(|| **CHAIN_ID.load());

        Ok(UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Legacy(tx),
            signature: Some(SignatureComponents::rlp_decode(r, 6, Some(v))?),
            chain_id:  id,
            hash:      Hasher::digest(r.as_raw()),
        })
    }
}

impl Eip2930Transaction {
    fn rlp_encode(
        &self,
        rlp: &mut RlpStream,
        chain_id: Option<u64>,
        signature: Option<&SignatureComponents>,
    ) {
        let rlp_stream_len = if signature.is_some() { 11 } else { 8 };
        rlp.begin_list(rlp_stream_len)
            .append(&(if let Some(id) = chain_id { id } else { 0 }))
            .append(&self.nonce)
            .append(&self.gas_price)
            .append(&self.gas_limit)
            .append(&self.action)
            .append(&self.value)
            .append(&self.data);

        rlp.begin_list(self.access_list.len());
        for access in self.access_list.iter() {
            rlp.begin_list(2);
            rlp.append(&access.address);
            rlp.begin_list(access.storage_keys.len());
            for storage_key in access.storage_keys.iter() {
                rlp.append(storage_key);
            }
        }

        if let Some(sig) = signature {
            sig.rlp_append(rlp);
        }
    }

    fn rlp_decode(r: &Rlp) -> Result<UnverifiedTransaction, DecoderError> {
        if r.item_count()? != 11 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        let id: u64 = r.val_at(0)?;
        let tx = Eip2930Transaction {
            nonce:       r.val_at(1)?,
            gas_price:   r.val_at(2)?,
            gas_limit:   r.val_at(3)?,
            action:      r.val_at(4)?,
            value:       r.val_at(5)?,
            data:        r.val_at(6)?,
            access_list: {
                let accl_rlp = r.at(7)?;
                let mut access_list: AccessList = Vec::new();
                for i in 0..accl_rlp.item_count()? {
                    let accounts = accl_rlp.at(i)?;
                    if accounts.item_count()? != 2 {
                        return Err(DecoderError::Custom("Unknown access list length"));
                    }

                    access_list.push(AccessListItem {
                        address:      accounts.val_at(0)?,
                        storage_keys: accounts.list_at(1)?,
                    });
                }
                access_list
            },
        };

        Ok(UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Eip2930(tx),
            signature: Some(SignatureComponents::rlp_decode(r, 8, None)?),
            chain_id:  id,
            hash:      Hasher::digest(&r.as_raw()),
        })
    }
}

impl Eip1559Transaction {
    fn rlp_encode(
        &self,
        rlp: &mut RlpStream,
        chain_id: Option<u64>,
        signature: Option<&SignatureComponents>,
    ) {
        let rlp_stream_len = if signature.is_some() { 12 } else { 9 };
        rlp.begin_list(rlp_stream_len)
            .append(&(if let Some(id) = chain_id { id } else { 0 }))
            .append(&self.nonce)
            .append(&self.max_priority_fee_per_gas)
            .append(&self.gas_price)
            .append(&self.gas_limit)
            .append(&self.action)
            .append(&self.value)
            .append(&self.data);

        rlp.begin_list(self.access_list.len());
        for access in self.access_list.iter() {
            rlp.begin_list(2);
            rlp.append(&access.address);
            rlp.begin_list(access.storage_keys.len());
            for storage_key in access.storage_keys.iter() {
                rlp.append(storage_key);
            }
        }

        if let Some(sig) = signature {
            sig.rlp_append(rlp);
        }
    }

    fn rlp_decode(r: &Rlp) -> Result<UnverifiedTransaction, DecoderError> {
        if r.item_count()? != 12 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        let id: u64 = r.val_at(0)?;
        let tx = Eip1559Transaction {
            nonce:                    r.val_at(1)?,
            max_priority_fee_per_gas: r.val_at(2)?,
            gas_price:                r.val_at(3)?,
            gas_limit:                r.val_at(4)?,
            action:                   r.val_at(5)?,
            value:                    r.val_at(6)?,
            data:                     r.val_at(7)?,
            access_list:              {
                let accl_rlp = r.at(8)?;
                let mut access_list: AccessList = Vec::new();
                for i in 0..accl_rlp.item_count()? {
                    let accounts = accl_rlp.at(i)?;
                    if accounts.item_count()? != 2 {
                        return Err(DecoderError::Custom("Unknown access list length"));
                    }

                    access_list.push(AccessListItem {
                        address:      accounts.val_at(0)?,
                        storage_keys: accounts.list_at(1)?,
                    });
                }
                access_list
            },
        };

        Ok(UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Eip1559(tx),
            signature: Some(SignatureComponents::rlp_decode(r, 9, None)?),
            chain_id:  id,
            hash:      Hasher::digest(&r.as_raw()),
        })
    }
}

impl Encodable for UnverifiedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        let chain_id = Some(self.chain_id);

        match &self.unsigned {
            UnsignedTransaction::Legacy(tx) => tx.rlp_encode(s, chain_id, self.signature.as_ref()),
            UnsignedTransaction::Eip2930(tx) => tx.rlp_encode(s, chain_id, self.signature.as_ref()),
            UnsignedTransaction::Eip1559(tx) => tx.rlp_encode(s, chain_id, self.signature.as_ref()),
        };
    }

    fn rlp_bytes(&self) -> BytesMut {
        let mut ret = BytesMut::new();
        let mut s = RlpStream::new();
        self.rlp_append(&mut s);

        match self.unsigned {
            UnsignedTransaction::Eip2930(_) => ret.put_u8(0x01),
            UnsignedTransaction::Eip1559(_) => ret.put_u8(0x02),
            _ => (),
        };

        ret.put(s.as_raw());
        ret
    }
}

impl Decodable for UnverifiedTransaction {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        let raw = r.as_raw();
        let header = raw[0];

        if (header & 0x80) != 0x00 {
            return LegacyTransaction::rlp_decode(r);
        }

        match header {
            0x01 => Eip2930Transaction::rlp_decode(&Rlp::new(&raw[1..])),
            0x02 => Eip1559Transaction::rlp_decode(&Rlp::new(&raw[1..])),
            _ => Err(DecoderError::Custom("Invalid transaction header")),
        }
    }
}

impl Encodable for SignedTransaction {
    fn rlp_append(&self, _s: &mut RlpStream) {}

    fn rlp_bytes(&self) -> BytesMut {
        self.transaction.rlp_bytes()
    }
}

impl Decodable for SignedTransaction {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        let utx = UnverifiedTransaction::decode(r)?;
        let public = Public::from_slice(
            &secp256k1_recover(
                utx.signature_hash().as_bytes(),
                utx.signature
                    .as_ref()
                    .ok_or(DecoderError::Custom("missing signature"))?
                    .as_bytes()
                    .as_ref(),
            )
            .map_err(|_| DecoderError::Custom("recover signature"))?
            .serialize_uncompressed()[1..65],
        );

        Ok(SignedTransaction {
            transaction: utx,
            sender:      public_to_address(&public),
            public:      Some(public),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rand::random;

    use common_crypto::secp256k1_recover;

    use crate::codec::hex_decode;
    use crate::types::{Bytes, Public, TransactionAction, H160, H256, U256};

    fn rand_bytes(len: usize) -> Bytes {
        Bytes::from((0..len).map(|_| random::<u8>()).collect::<Vec<_>>())
    }

    fn mock_eip1559_transaction() -> Eip1559Transaction {
        Eip1559Transaction {
            nonce:                    U256::one(),
            gas_limit:                U256::one(),
            max_priority_fee_per_gas: U256::one(),
            gas_price:                U256::one(),
            action:                   TransactionAction::Create,
            value:                    U256::one(),
            data:                     rand_bytes(32).to_vec().into(),
            access_list:              vec![],
        }
    }

    fn mock_sig_component() -> SignatureComponents {
        SignatureComponents {
            standard_v: 4,
            r:          Bytes::default(),
            s:          Bytes::default(),
        }
    }

    fn mock_unverfied_tx() -> UnverifiedTransaction {
        UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Eip1559(mock_eip1559_transaction()),
            chain_id:  random::<u64>(),
            hash:      H256::default(),
            signature: Some(mock_sig_component()),
        }
        .calc_hash()
    }

    fn mock_signed_tx() -> SignedTransaction {
        SignedTransaction {
            transaction: mock_unverfied_tx(),
            sender:      H160::default(),
            public:      None,
        }
    }

    #[test]
    fn test_decode_sender() {
        let bytes = hex_decode("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a801ba048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap();
        let tx = UnverifiedTransaction::decode(&Rlp::new(&bytes)).unwrap();

        assert!(tx.unsigned.data().is_empty());
        assert_eq!(tx.unsigned.gas_limit(), U256::from(0x5208u64));
        assert_eq!(tx.unsigned.gas_price(), U256::from(0x01u64));
        assert_eq!(tx.unsigned.nonce(), U256::from(0x00u64));
        assert_eq!(
            tx.unsigned.to().unwrap(),
            H160::from_slice(&hex_decode("095e7baea6a6c7c4c2dfeb977efac326af552d87").unwrap())
        );
        assert_eq!(tx.unsigned.value(), U256::from(0x0au64));
        assert_eq!(
            public_to_address(&tx.recover_public().unwrap()),
            H160::from_slice(&hex_decode("0f65fe9276bc9a24ae7083ae28e2660ef72df99e").unwrap())
        );
        assert_eq!(tx.chain_id, 0);
    }

    #[test]
    fn test_signed_tx_codec() {
        let raw = hex_decode("02f8670582010582012c82012c825208945cf83df52a32165a7f392168ac009b168c9e89150180c001a0a68aeb0db4d84cf16da5a6918becefd254654854cfc23f0112ef78154ce84db89f4b0af1cbf12f5bfaec81c3d4d495717d720b574a05092f6b436c2ab255cd35").unwrap();
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw)).unwrap();
        let origin: SignedTransaction = utx.try_into().unwrap();
        let encode = origin.rlp_bytes().freeze().to_vec();
        let decode: SignedTransaction = rlp::decode(&encode).unwrap();
        assert_eq!(origin, decode);
    }

    #[test]
    fn test_decode_unsigned_tx() {
        let raw = hex_decode("02f9016e2a80830f4240830f4240825208948d97689c9818892b700e27f316cc3e41e17fbeb9872386f26fc10000b8fe608060405234801561001057600080fd5b5060df8061001f6000396000f3006080604052600436106049576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806360fe47b114604e5780636d4ce63c146078575b600080fd5b348015605957600080fd5b5060766004803603810190808035906020019092919050505060a0565b005b348015608357600080fd5b50608a60aa565b6040518082815260200191505060405180910390f35b8060008190555050565b600080549050905600a165627a7a7230582099c66a25d59f0aa78f7ebc40748fa1d1fbc335d8d780f284841b30e0365acd960029c001a055ea090c41cb5c76a7065a04fc6355d7804809baccc8f86717ac4da1694621fba03310f10f3488b558f65a94fc164036aa69d88ab35f42dcf5d77b6f04c5cf8e72").unwrap();
        let rlp = Rlp::new(&raw);
        let res = UnverifiedTransaction::decode(&rlp);
        assert!(res.is_ok());
    }

    #[test]
    fn test_decode_unverified_tx() {
        let raw = hex_decode("02f8670582010582012c82012c825208945cf83df52a32165a7f392168ac009b168c9e89150180c001a0a68aeb0db4d84cf16da5a6918becefd254654854cfc23f0112ef78154ce84db89f4b0af1cbf12f5bfaec81c3d4d495717d720b574a05092f6b436c2ab255cd35").unwrap();
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw)).unwrap();
        let _public = Public::from_slice(
            &secp256k1_recover(
                utx.hash.as_bytes(),
                utx.signature.as_ref().unwrap().as_bytes().as_ref(),
            )
            .unwrap()
            .serialize_uncompressed()[1..65],
        );

        let sig = utx.signature.unwrap();
        assert_ne!(sig.s, sig.r);
    }

    #[test]
    fn test_calc_tx_hash() {
        let raw = hex_decode("02f8690505030382520894a15da349978753d846eede580c7de8e590c1e5b8872386f26fc1000080c080a097d7a69ce423c2a5814daf71345b49698db5839e092f744e263983b56a992b87a02a5e12966dccbc8e3f6f21ffb528372c915c202381cfcbe3b8cf8ef8af273e99").unwrap();
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw)).unwrap();
        let hash = utx.calc_hash().hash;
        assert_eq!(
            hash.as_bytes(),
            hex_decode("4c6d0ffa15709084a4b2b546f32503e4ccf2fb26b6c894df773b2d14b7c96e3f").unwrap()
        );
    }
}
