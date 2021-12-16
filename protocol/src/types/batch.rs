use crate::types::{Block, Bytes, SignedTransaction};

macro_rules! batch_msg_type {
    ($name: ident, $ty: ident) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name(pub Vec<$ty>);

        impl crate::traits::MessageCodec for $name {
            fn encode_msg(&mut self) -> crate::ProtocolResult<Bytes> {
                let bytes = rlp::encode_list(&self.0);
                Ok(bytes.freeze())
            }

            fn decode_msg(bytes: Bytes) -> crate::ProtocolResult<Self> {
                let inner: Vec<$ty> = rlp::Rlp::new(bytes.as_ref())
                    .as_list()
                    .map_err(|e| crate::codec::error::CodecError::Rlp(e.to_string()))?;
                Ok(Self(inner))
            }
        }

        impl $name {
            pub fn inner(self) -> Vec<$ty> {
                self.0
            }
        }
    };
}

batch_msg_type!(BatchSignedTxs, SignedTransaction);
batch_msg_type!(BatchBlocks, Block);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::ProtocolCodec;
    use crate::types::{
        SignatureComponents, Transaction, TransactionAction, UnverifiedTransaction, U256,
    };
    use rand::random;

    fn mock_sign_tx() -> SignedTransaction {
        let utx = UnverifiedTransaction {
            unsigned:  Transaction {
                nonce:                    U256::one(),
                max_priority_fee_per_gas: U256::one(),
                gas_price:                U256::one(),
                gas_limit:                U256::one(),
                action:                   TransactionAction::Create,
                value:                    Default::default(),
                data:                     Bytes::new(),
                access_list:              vec![],
            },
            signature: Some(SignatureComponents {
                standard_v: 4,
                r:          Default::default(),
                s:          Default::default(),
            }),
            chain_id:  random::<u64>(),
            hash:      Default::default(),
        };
        let utx = utx.hash();

        SignedTransaction {
            transaction: utx,
            sender:      Default::default(),
            public:      Default::default(),
        }
    }

    #[test]
    fn test_codec() {
        let stx = mock_sign_tx();
        let raw = rlp::encode(&stx);
        let decode = SignedTransaction::decode(raw).unwrap();
        assert_eq!(stx, decode);
    }
}
