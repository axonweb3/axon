use bytes::BufMut;
use common_crypto::secp256k1_recover;
use ethereum_types::BigEndianHash;
use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{
    public_to_address, AccessList, AccessListItem, Bytes, BytesMut, Public, SignatureComponents,
    SignedTransaction, Transaction, TransactionAction, UnverifiedTransaction, H256, U256,
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

impl Decodable for SignatureComponents {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        let standard_v: u8 = r.val_at(0)?;
        let r_: Bytes = r.val_at(1)?;
        let s: Bytes = r.val_at(2)?;

        Ok(SignatureComponents {
            standard_v,
            r: r_,
            s,
        })
    }
}

impl Encodable for UnverifiedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        let rlp_stream_len = if self.signature.is_some() {
            12usize
        } else {
            9usize
        };

        s.begin_list(rlp_stream_len)
            .append(&self.chain_id)
            .append(&self.unsigned.nonce)
            .append(&self.unsigned.max_priority_fee_per_gas)
            .append(&self.unsigned.gas_price)
            .append(&self.unsigned.gas_limit)
            .append(&self.unsigned.action)
            .append(&self.unsigned.value)
            .append(&self.unsigned.data);
        s.begin_list(self.unsigned.access_list.len());
        for access in self.unsigned.access_list.iter() {
            s.begin_list(2);
            s.append(&access.address);
            s.begin_list(access.storage_keys.len());
            for storage_key in access.storage_keys.iter() {
                s.append(storage_key);
            }
        }

        if let Some(signature) = &self.signature {
            signature.rlp_append(s);
        }
    }

    fn rlp_bytes(&self) -> BytesMut {
        let mut ret = BytesMut::new();
        let mut s = RlpStream::new();
        self.rlp_append(&mut s);
        ret.put_u8(0x02);
        ret.put(s.as_raw());
        ret
    }
}

impl Decodable for UnverifiedTransaction {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        if r.item_count()? != 12 {
            return Err(DecoderError::RlpIncorrectListLen);
        }

        let chain_id: u64 = r.val_at(0)?;
        let nonce: U256 = r.val_at(1)?;
        let max_priority_fee_per_gas: U256 = r.val_at(2)?;
        let gas_price: U256 = r.val_at(3)?;
        let gas_limit: U256 = r.val_at(4)?;
        let action: TransactionAction = r.val_at(5)?;
        let value: U256 = r.val_at(6)?;
        let data: Bytes = r.val_at(7)?;

        // access list we get from here
        let accl_rlp = r.at(8)?;

        // access_list pattern: [[{20 bytes}, [{32 bytes}...]]...]
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

        let v: u8 = r.val_at(9)?;
        let eth_tx_flag = v <= 1;
        let signature = SignatureComponents {
            standard_v: v,
            r:          if eth_tx_flag {
                let tmp: U256 = r.val_at(10)?;
                Bytes::from(<H256 as BigEndianHash>::from_uint(&tmp).as_bytes().to_vec())
            } else {
                r.val_at(10)?
            },
            s:          if eth_tx_flag {
                let tmp: U256 = r.val_at(11)?;
                Bytes::from(<H256 as BigEndianHash>::from_uint(&tmp).as_bytes().to_vec())
            } else {
                r.val_at(11)?
            },
        };

        let utx = UnverifiedTransaction {
            unsigned: Transaction {
                nonce,
                max_priority_fee_per_gas,
                gas_price,
                gas_limit,
                action,
                value,
                data,
                access_list,
            },
            hash: Default::default(),
            signature: Some(signature),
            chain_id,
        };

        println!("sig hash {:?}", utx.signature_hash());

        Ok(utx.calc_hash())
    }
}

impl Encodable for SignedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(1).append(&self.transaction);
    }
}

impl Decodable for SignedTransaction {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(1) => {
                let utx: UnverifiedTransaction = r.val_at(0)?;
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
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
        }
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

    fn mock_transaction() -> Transaction {
        Transaction {
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
            unsigned:  mock_transaction(),
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
    fn test_signed_tx_codec() {
        let raw = hex_decode("02f8670582010582012c82012c825208945cf83df52a32165a7f392168ac009b168c9e89150180c001a0a68aeb0db4d84cf16da5a6918becefd254654854cfc23f0112ef78154ce84db89f4b0af1cbf12f5bfaec81c3d4d495717d720b574a05092f6b436c2ab255cd35").unwrap();
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw[1..])).unwrap();
        let origin: SignedTransaction = utx.try_into().unwrap();
        let encode = origin.rlp_bytes().freeze().to_vec();
        let decode: SignedTransaction = rlp::decode(&encode).unwrap();
        assert_eq!(origin, decode);
    }

    #[test]
    fn test_decode_unsigned_tx() {
        let raw = hex_decode("02f9016e2a80830f4240830f4240825208948d97689c9818892b700e27f316cc3e41e17fbeb9872386f26fc10000b8fe608060405234801561001057600080fd5b5060df8061001f6000396000f3006080604052600436106049576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806360fe47b114604e5780636d4ce63c146078575b600080fd5b348015605957600080fd5b5060766004803603810190808035906020019092919050505060a0565b005b348015608357600080fd5b50608a60aa565b6040518082815260200191505060405180910390f35b8060008190555050565b600080549050905600a165627a7a7230582099c66a25d59f0aa78f7ebc40748fa1d1fbc335d8d780f284841b30e0365acd960029c001a055ea090c41cb5c76a7065a04fc6355d7804809baccc8f86717ac4da1694621fba03310f10f3488b558f65a94fc164036aa69d88ab35f42dcf5d77b6f04c5cf8e72").unwrap();
        let rlp = Rlp::new(&raw[1..]);
        let res = UnverifiedTransaction::decode(&rlp);
        assert!(res.is_ok());
    }

    #[test]
    fn test_decode_unverified_tx() {
        let raw = hex_decode("02f8670582010582012c82012c825208945cf83df52a32165a7f392168ac009b168c9e89150180c001a0a68aeb0db4d84cf16da5a6918becefd254654854cfc23f0112ef78154ce84db89f4b0af1cbf12f5bfaec81c3d4d495717d720b574a05092f6b436c2ab255cd35").unwrap();
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw[1..])).unwrap();
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
        let utx = UnverifiedTransaction::decode(&Rlp::new(&raw[1..])).unwrap();
        let hash = utx.calc_hash().hash;
        assert_eq!(
            hash.as_bytes(),
            hex_decode("4c6d0ffa15709084a4b2b546f32503e4ccf2fb26b6c894df773b2d14b7c96e3f").unwrap()
        );
    }
}
