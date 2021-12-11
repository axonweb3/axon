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

        // impl crate::codec::ProtocolCodec for $name {
        //     fn encode(&self) -> crate::ProtocolResult<Bytes> {
        //         self.encode_msg()
        //     }

        //     fn decode<B: AsRef<[u8]>>(bytes: B) -> crate::ProtocolResult<Self> {
        //         Self::decode_msg(bytes)
        //     }
        // }

        impl $name {
            pub fn inner(self) -> Vec<$ty> {
                self.0
            }
        }
    };
}

batch_msg_type!(BatchSignedTxs, SignedTransaction);
batch_msg_type!(BatchBlocks, Block);
