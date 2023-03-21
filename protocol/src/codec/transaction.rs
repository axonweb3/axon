use bytes::BufMut;
use ethereum_types::BigEndianHash;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use common_crypto::secp256k1_recover;

use crate::types::{
    public_to_address, AccessList, AccessListItem, Bytes, BytesMut, Eip1559Transaction,
    Eip2930Transaction, Hasher, LegacyTransaction, Public, SignatureComponents, SignedTransaction,
    UnsignedTransaction, UnverifiedTransaction, H256, U256,
};

fn truncate_slice<T>(s: &[T], n: usize) -> &[T] {
    match s.len() {
        l if l <= n => s,
        _ => &s[0..n],
    }
}

impl Encodable for SignatureComponents {
    fn rlp_append(&self, s: &mut RlpStream) {
        if self.is_eth_sig() {
            let r = U256::from(truncate_slice(&self.r, 32));
            let s_ = U256::from(truncate_slice(&self.s, 32));
            s.append(&self.standard_v).append(&r).append(&s_);
        } else {
            s.append(&self.standard_v).append(&self.r).append(&self.s);
        }
    }
}

impl SignatureComponents {
    fn rlp_decode(rlp: &Rlp, offset: usize, legacy_v: Option<u64>) -> Result<Self, DecoderError> {
        let v: u8 = if let Some(n) = legacy_v {
            SignatureComponents::extract_standard_v(n)
                .ok_or(DecoderError::Custom("invalid legacy v in signature"))?
        } else {
            rlp.val_at(offset)?
        };

        let signature_r_offset = offset + 1;
        let signature_s_offset = offset + 2;
        let signature_r_and_s_size =
            rlp.at(signature_r_offset)?.size() + rlp.at(signature_s_offset)?.size();

        let eth_tx_flag = signature_r_and_s_size <= 64;
        let (r, s) = match eth_tx_flag {
            true => {
                let tmp_r: U256 = rlp.val_at(signature_r_offset)?;
                let tmp_s: U256 = rlp.val_at(signature_s_offset)?;
                (
                    Bytes::from(
                        <H256 as BigEndianHash>::from_uint(&tmp_r)
                            .as_bytes()
                            .to_vec(),
                    ),
                    Bytes::from(
                        <H256 as BigEndianHash>::from_uint(&tmp_s)
                            .as_bytes()
                            .to_vec(),
                    ),
                )
            }
            false => (
                rlp.val_at(signature_r_offset)?,
                rlp.val_at(signature_s_offset)?,
            ),
        };

        Ok(SignatureComponents {
            standard_v: v,
            r,
            s,
        })
    }
}

impl LegacyTransaction {
    pub fn rlp_encode(
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
            rlp.append(&sig.add_chain_replay_protection(chain_id));

            if sig.is_eth_sig() {
                rlp.append(&U256::from(truncate_slice(&sig.r, 32)))
                    .append(&U256::from(truncate_slice(&sig.s, 32)));
            } else {
                rlp.append(&sig.r).append(&sig.s);
            }
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

        Ok(UnverifiedTransaction {
            unsigned:  UnsignedTransaction::Legacy(tx),
            signature: Some(SignatureComponents::rlp_decode(r, 6, Some(v))?),
            chain_id:  SignatureComponents::extract_chain_id(v)
                .ok_or(DecoderError::Custom("Missing chain id"))?,
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
        let tx = UnsignedTransaction::Eip2930(Eip2930Transaction {
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
        });

        Ok(UnverifiedTransaction {
            hash:      Hasher::digest([&[tx.as_u8()], r.as_raw()].concat()),
            unsigned:  tx,
            signature: Some(SignatureComponents::rlp_decode(r, 8, None)?),
            chain_id:  id,
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
        let tx = UnsignedTransaction::Eip1559(Eip1559Transaction {
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
        });

        Ok(UnverifiedTransaction {
            hash:      Hasher::digest([&[tx.as_u8()], r.as_raw()].concat()),
            unsigned:  tx,
            signature: Some(SignatureComponents::rlp_decode(r, 9, None)?),
            chain_id:  id,
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

        if !self.unsigned.is_legacy() {
            ret.put_u8(self.unsigned.as_u8());
        }

        ret.put(s.out());
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
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append(&self.transaction.rlp_bytes());
    }
}

impl Decodable for SignedTransaction {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        let utx = UnverifiedTransaction::decode(&Rlp::new(r.data()?))?;
        let sig = utx
            .signature
            .as_ref()
            .ok_or(DecoderError::Custom("missing signature"))?;

        let public = if sig.is_eth_sig() {
            Public::from_slice(
                &secp256k1_recover(utx.signature_hash(true).as_bytes(), sig.as_bytes().as_ref())
                    .map_err(|_| DecoderError::Custom("recover signature"))?
                    .serialize_uncompressed()[1..65],
            )
        } else {
            Public::zero()
        };

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

    fn mock_unverified_tx() -> UnverifiedTransaction {
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
            transaction: mock_unverified_tx(),
            sender:      H160::default(),
            public:      None,
        }
    }

    #[test]
    fn test_legacy_decode() {
        let bytes = hex_decode("f85f800182520894095e7baea6a6c7c4c2dfeb977efac326af552d870a8023a048b55bfa915ac795c431978d8a6a992b628d557da5ff759b307d495a36649353a0efffd310ac743f371de3b9f7f9cb56c0b28ad43601b4ab949f53faa07bd2c804").unwrap();
        let tx = UnverifiedTransaction::decode(&Rlp::new(&bytes)).unwrap();

        assert!(tx.unsigned.data().is_empty());
        assert_eq!(*tx.unsigned.gas_limit(), U256::from(0x5208u64));
        assert_eq!(tx.unsigned.gas_price(), U256::from(0x01u64));
        assert_eq!(*tx.unsigned.nonce(), U256::from(0x00u64));
        assert_eq!(
            tx.unsigned.to().unwrap(),
            H160::from_slice(&hex_decode("095e7baea6a6c7c4c2dfeb977efac326af552d87").unwrap())
        );
        assert_eq!(*tx.unsigned.value(), U256::from(0x0au64));
        assert_eq!(
            public_to_address(&tx.recover_public(false).unwrap()),
            H160::from_slice(&hex_decode("0f65fe9276bc9a24ae7083ae28e2660ef72df99e").unwrap())
        );
        assert_eq!(tx.chain_id, 0);
        assert!(tx.signature.unwrap().is_eth_sig());
    }

    #[test]
    fn test_signed_tx_codec() {
        let raw = hex_decode("02f8670582010582012c82012c825208945cf83df52a32165a7f392168ac009b168c9e89150180c001a0a68aeb0db4d84cf16da5a6918becefd254654854cfc23f0112ef78154ce84db89f4b0af1cbf12f5bfaec81c3d4d495717d720b574a05092f6b436c2ab255cd35").unwrap();
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw)).unwrap();
        let origin = SignedTransaction::from_unverified(utx, None).unwrap();
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
    fn test_legacy_encode() {
        let raw = hex_decode("f885020883011493941c85638e118b37167e9298c2268758e058ddfda08203e8a4f9846e1f00000000000000000000000000000000000000000000000000000000000000012da05595614cb1397fb947b3512af6939c1704c85b49c9ab8c16121e12073350b4ca9fd08cd623473664607cbe7d13dbb11a44f06ad8ce499e585ef91929b6b6e2e7").unwrap();
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw)).unwrap();

        let encode = utx.rlp_bytes();
        let recover = UnverifiedTransaction::decode(&Rlp::new(&encode)).unwrap();

        assert_eq!(utx, recover);
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
        assert!(sig.is_eth_sig());
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

    #[test]
    fn should_agree_with_vitalik() {
        let test_vector = |tx_data: &str, address: &'static str| {
            let utx =
                UnverifiedTransaction::decode(&Rlp::new(&hex_decode(tx_data).unwrap())).unwrap();
            let signed = SignedTransaction::from_unverified(utx.clone(), None).unwrap();
            assert_eq!(
                signed.sender,
                H160::from_slice(&hex_decode(address).unwrap())
            );
            assert!(utx.signature.unwrap().is_eth_sig());
        };

        test_vector("f864808504a817c800825208943535353535353535353535353535353535353535808025a0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116da0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d", "f0f6f18bca1b28cd68e4357452947e021241e9ce");
        test_vector("f864018504a817c80182a410943535353535353535353535353535353535353535018025a0489efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bcaa0489efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6", "23ef145a395ea3fa3deb533b8a9e1b4c6c25d112");
        test_vector("f864028504a817c80282f618943535353535353535353535353535353535353535088025a02d7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5a02d7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5", "2e485e0c23b4c3c542628a5f672eeab0ad4888be");
        test_vector("f865038504a817c803830148209435353535353535353535353535353535353535351b8025a02a80e1ef1d7842f27f2e6be0972bb708b9a135c38860dbe73c27c3486c34f4e0a02a80e1ef1d7842f27f2e6be0972bb708b9a135c38860dbe73c27c3486c34f4de", "82a88539669a3fd524d669e858935de5e5410cf0");
        test_vector("f865048504a817c80483019a28943535353535353535353535353535353535353535408025a013600b294191fc92924bb3ce4b969c1e7e2bab8f4c93c3fc6d0a51733df3c063a013600b294191fc92924bb3ce4b969c1e7e2bab8f4c93c3fc6d0a51733df3c060", "f9358f2538fd5ccfeb848b64a96b743fcc930554");
        test_vector("f865058504a817c8058301ec309435353535353535353535353535353535353535357d8025a04eebf77a833b30520287ddd9478ff51abbdffa30aa90a8d655dba0e8a79ce0c1a04eebf77a833b30520287ddd9478ff51abbdffa30aa90a8d655dba0e8a79ce0c1", "a8f7aba377317440bc5b26198a363ad22af1f3a4");
        test_vector("f866068504a817c80683023e3894353535353535353535353535353535353535353581d88025a06455bf8ea6e7463a1046a0b52804526e119b4bf5136279614e0b1e8e296a4e2fa06455bf8ea6e7463a1046a0b52804526e119b4bf5136279614e0b1e8e296a4e2d", "f1f571dc362a0e5b2696b8e775f8491d3e50de35");
        test_vector("f867078504a817c807830290409435353535353535353535353535353535353535358201578025a052f1a9b320cab38e5da8a8f97989383aab0a49165fc91c737310e4f7e9821021a052f1a9b320cab38e5da8a8f97989383aab0a49165fc91c737310e4f7e9821021", "d37922162ab7cea97c97a87551ed02c9a38b7332");
        test_vector("f867088504a817c8088302e2489435353535353535353535353535353535353535358202008025a064b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c12a064b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c10", "9bddad43f934d313c2b79ca28a432dd2b7281029");
        test_vector("f867098504a817c809830334509435353535353535353535353535353535353535358202d98025a052f8f61201b2b11a78d6e866abc9c3db2ae8631fa656bfe5cb53668255367afba052f8f61201b2b11a78d6e866abc9c3db2ae8631fa656bfe5cb53668255367afb", "3c24d7329e92f84f08556ceb6df1cdb0104ca49f");
    }

    #[test]
    fn test_interoperation_tx_codec() {
        let raw_tx = hex::decode("f901ed80808094cb9112d826471e7deb7bc895b1771e5d676a14af880de0b6b3a7640000802db86302f860e3a0f35178c7a1a5a4e5b164157aa549a493cebc9a3079b6a9ede7ae5207adb3f4d48001c0f839a0d23761b364210735c19c60561d213fb3beae2fd6172743719eff6920e020baac9600016091d93dbab12f16640fb3a0a8f1e77e03fbc51c02b90162f9015ff9015cb90157014599a5795423d54ab8e1f44f5c6ef5be9b1829beddb787bc732e4469d25f8c93e94afa393617f905bf1765c35dc38501a862b4b2f794a88b4f9010da02411a852d147a369b9ba6de71bf065f4831cc1ff9c4887c2dcfa669d6e4b9d24f0937c154974fd8399405052fdc8a6605a86040d670d47db1a092916aa5679b2e8604b449960de5880e8c687434170f6476605b8fe4aeb9a28632c7995cf3ba831d97630162f9fb777b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a22593255355a544d324d545135595445775a5745345a6a64684e6a4a694e47526c4d574d7a4d5755784f44686c4e6a597a4d5745784d6d46685a44566d597a426a596d4e6d4f4746694d6a45774f4751334d6a646d4f51222c226f726967696e223a22687474703a2f2f6c6f63616c686f73743a38303030222c2263726f73734f726967696e223a66616c73657dc0c0").unwrap();
        let rlp = Rlp::new(&raw_tx);
        let utx = UnverifiedTransaction::decode(&rlp).unwrap().calc_hash();
        let sig = utx.signature.unwrap();

        assert_eq!(
            utx.unsigned.to().unwrap(),
            H160::from_slice(&hex_decode("Cb9112D826471E7DEB7Bc895b1771e5d676a14AF").unwrap())
        );
        assert!(utx.unsigned.data().is_empty());
        assert!(utx.unsigned.nonce().is_zero());
        assert!(utx.unsigned.gas_limit().is_zero());
        assert!(utx.unsigned.gas_price().is_zero());
        assert!(utx.unsigned.max_priority_fee_per_gas().is_zero());
        assert!(utx.chain_id == 5);
        assert_eq!(sig.standard_v, 0);
        assert_eq!(sig.r, hex_decode("02f860e3a0f35178c7a1a5a4e5b164157aa549a493cebc9a3079b6a9ede7ae5207adb3f4d48001c0f839a0d23761b364210735c19c60561d213fb3beae2fd6172743719eff6920e020baac9600016091d93dbab12f16640fb3a0a8f1e77e03fbc51c02").unwrap());
        assert_eq!(sig.s, hex_decode("0xf9015ff9015cb90157014599a5795423d54ab8e1f44f5c6ef5be9b1829beddb787bc732e4469d25f8c93e94afa393617f905bf1765c35dc38501a862b4b2f794a88b4f9010da02411a852d147a369b9ba6de71bf065f4831cc1ff9c4887c2dcfa669d6e4b9d24f0937c154974fd8399405052fdc8a6605a86040d670d47db1a092916aa5679b2e8604b449960de5880e8c687434170f6476605b8fe4aeb9a28632c7995cf3ba831d97630162f9fb777b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a22593255355a544d324d545135595445775a5745345a6a64684e6a4a694e47526c4d574d7a4d5755784f44686c4e6a597a4d5745784d6d46685a44566d597a426a596d4e6d4f4746694d6a45774f4751334d6a646d4f51222c226f726967696e223a22687474703a2f2f6c6f63616c686f73743a38303030222c2263726f73734f726967696e223a66616c73657dc0c0").unwrap());
    }

    #[test]
    fn test_secp256r1_sig_decode() {
        let raw = hex_decode("f901f5808408653b0282520894cb9112d826471e7deb7bc895b1771e5d676a14af880de0b6b3a764000080820fefb86302f860e3a0f35178c7a1a5a4e5b164157aa549a493cebc9a3079b6a9ede7ae5207adb3f4d48001c0f839a0d23761b364210735c19c60561d213fb3beae2fd6172743719eff6920e020baac9600016091d93dbab12f16640fb3a0a8f1e77e03fbc51c02b90162f9015ff9015cb90157014599a5795423d54ab8e1f44f5c6ef5be9b1829beddb787bc732e4469d25f8c93e94afa393617f905bf1765c35dc38501a862b4b2f794a88b4f9010da02411a85754d08b9c62ce935f505b478662953815be16f40f19bcb55236713180a697ceac060a7b05bb55c6dcd249813b5bd9f1f295a038c9d5980b201b3e538bfa30ddd49960de5880e8c687434170f6476605b8fe4aeb9a28632c7995cf3ba831d97630162f9fb777b2274797065223a22776562617574686e2e676574222c226368616c6c656e6765223a22596a4e6d597a41355a6a63775a574d794e324e6d5a54417959544d784d4451794d4445354d47557a4f545a6b596a4a6d5a6a6b78596a49775a444e6d4e3255314f4755354d7a49354e6a52684e445a695a54566c5a67222c226f726967696e223a22687474703a2f2f6c6f63616c686f73743a38303030222c2263726f73734f726967696e223a66616c73657dc0c0").unwrap();
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw)).unwrap();
        assert!(utx.check_hash().is_ok());
        assert!(!utx.signature.unwrap().is_eth_sig());
    }
}
