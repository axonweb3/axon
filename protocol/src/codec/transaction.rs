use rlp::{Decodable, DecoderError, Encodable, Prototype, Rlp, RlpStream};

use crate::types::{
    Address, Public, SignatureComponents, SignedTransaction, Transaction, UnverifiedTransaction,
    H256,
};

impl Encodable for SignatureComponents {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3)
            .append(&self.r)
            .append(&self.s)
            .append(&self.standard_v);
    }
}

impl Decodable for SignatureComponents {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(3) => {
                let r_: H256 = r.val_at(0)?;
                let s: H256 = r.val_at(1)?;
                let standard_v: u8 = r.val_at(2)?;

                Ok(SignatureComponents {
                    standard_v,
                    r: r_,
                    s,
                })
            }
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
        }
    }
}

impl Encodable for UnverifiedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(4)
            .append(&self.unsigned)
            .append(&self.signature)
            .append(&self.chain_id)
            .append(&self.hash);
    }
}

impl Decodable for UnverifiedTransaction {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(4) => {
                let unsigned: Transaction = r.val_at(0)?;
                let signature: SignatureComponents = r.val_at(1)?;
                let chain_id: Option<u64> = r.val_at(2)?;
                let hash: H256 = r.val_at(3)?;

                Ok(UnverifiedTransaction {
                    unsigned,
                    signature,
                    chain_id,
                    hash,
                })
            }
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
        }
    }
}

impl Encodable for SignedTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3)
            .append(&self.transaction)
            .append(&self.sender)
            .append(&self.public);
    }
}

impl Decodable for SignedTransaction {
    fn decode(r: &Rlp) -> Result<Self, DecoderError> {
        match r.prototype()? {
            Prototype::List(3) => {
                let transaction: UnverifiedTransaction = r.val_at(0)?;
                let sender: Address = r.val_at(1)?;
                let public: Public = r.val_at(2)?;

                Ok(SignedTransaction {
                    transaction,
                    sender,
                    public,
                })
            }
            _ => Err(DecoderError::RlpInconsistentLengthAndData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Bytes, TransactionAction, U256};
    use rand::random;

    fn rand_bytes(len: usize) -> Bytes {
        Bytes::from((0..len).map(|_| random::<u8>()).collect::<Vec<_>>())
    }

    fn mock_transaction() -> Transaction {
        Transaction {
            chain_id:                 random::<u64>(),
            nonce:                    U256::one(),
            gas_limit:                U256::one(),
            max_priority_fee_per_gas: U256::one(),
            max_fee_per_gas:          U256::one(),
            action:                   TransactionAction::Create,
            value:                    U256::one(),
            input:                    rand_bytes(32).to_vec(),
            access_list:              vec![],
            odd_y_parity:             true,
            r:                        H256::default(),
            s:                        H256::default(),
        }
    }

    fn mock_sig_component() -> SignatureComponents {
        SignatureComponents {
            standard_v: random::<u8>(),
            r:          H256::default(),
            s:          H256::default(),
        }
    }

    fn mock_unverfied_tx(chain_id: Option<u64>) -> UnverifiedTransaction {
        UnverifiedTransaction {
            unsigned: mock_transaction(),
            chain_id,
            hash: H256::default(),
            signature: mock_sig_component(),
        }
    }

    fn mock_signed_tx(has_chain_id: bool) -> SignedTransaction {
        SignedTransaction {
            transaction: if has_chain_id {
                mock_unverfied_tx(Some(random::<u64>()))
            } else {
                mock_unverfied_tx(None)
            },
            sender:      Address::default(),
            public:      Public::default(),
        }
    }

    #[test]
    fn test_signed_tx_codec() {
        let origin = mock_signed_tx(true);
        let encode = rlp::encode(&origin).freeze().to_vec();
        let decode: SignedTransaction = rlp::decode(&encode).unwrap();
        assert_eq!(origin, decode);

        let origin = mock_signed_tx(false);
        let encode = rlp::encode(&origin).freeze().to_vec();
        let decode: SignedTransaction = rlp::decode(&encode).unwrap();
        assert_eq!(origin, decode);
    }
}
