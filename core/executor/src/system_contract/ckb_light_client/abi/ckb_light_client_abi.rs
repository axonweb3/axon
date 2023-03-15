pub use ckb_light_client::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod ckb_light_client {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    use ethers::contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers::core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers::providers::Middleware;
    #[doc = "CkbLightClient was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes32[]\",\n        \"name\": \"blockHashs\",\n        \"type\": \"bytes32[]\"\n      }\n    ],\n    \"name\": \"rollback\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"allowRead\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"name\": \"setState\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"uint32\",\n            \"name\": \"version\",\n            \"type\": \"uint32\"\n          },\n          {\n            \"internalType\": \"uint32\",\n            \"name\": \"compactTarget\",\n            \"type\": \"uint32\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"timestamp\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"number\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"epoch\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"parentHash\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"transactionsRoot\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"proposalsHash\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"unclesHash\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"dao\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"uint128\",\n            \"name\": \"nonce\",\n            \"type\": \"uint128\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"blockHash\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"internalType\": \"struct CkbType.Header[]\",\n        \"name\": \"headers\",\n        \"type\": \"tuple[]\"\n      }\n    ],\n    \"name\": \"update\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static CKBLIGHTCLIENT_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct CkbLightClient<M>(ethers::contract::Contract<M>);
    impl<M> Clone for CkbLightClient<M> {
        fn clone(&self) -> Self {
            CkbLightClient(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for CkbLightClient<M> {
        type Target = ethers::contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for CkbLightClient<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(CkbLightClient))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> CkbLightClient<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), CKBLIGHTCLIENT_ABI.clone(), client)
                .into()
        }

        #[doc = "Calls the contract's `rollback` (0xd32a2285) function"]
        pub fn rollback(
            &self,
            block_hashs: ::std::vec::Vec<[u8; 32]>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([211, 42, 34, 133], block_hashs)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `setState` (0xac9f0222) function"]
        pub fn set_state(
            &self,
            allow_read: bool,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([172, 159, 2, 34], allow_read)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `update` (0x2b196cf3) function"]
        pub fn update(
            &self,
            headers: ::std::vec::Vec<Header>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([43, 25, 108, 243], headers)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for CkbLightClient<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `rollback` function with signature `rollback(bytes32[])` and selector `[211, 42, 34, 133]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers::contract::EthCall,
        ethers::contract::EthDisplay,
        Default,
    )]
    #[ethcall(name = "rollback", abi = "rollback(bytes32[])")]
    pub struct RollbackCall {
        pub block_hashs: ::std::vec::Vec<[u8; 32]>,
    }
    #[doc = "Container type for all input parameters for the `setState` function with signature `setState(bool)` and selector `[172, 159, 2, 34]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers::contract::EthCall,
        ethers::contract::EthDisplay,
        Default,
    )]
    #[ethcall(name = "setState", abi = "setState(bool)")]
    pub struct SetStateCall {
        pub allow_read: bool,
    }
    #[doc = "Container type for all input parameters for the `update` function with signature `update((uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,bytes32,bytes32,bytes32,uint128,bytes32)[])` and selector `[43, 25, 108, 243]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers::contract::EthCall,
        ethers::contract::EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "update",
        abi = "update((uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,bytes32,bytes32,bytes32,uint128,bytes32)[])"
    )]
    pub struct UpdateCall {
        pub headers: ::std::vec::Vec<Header>,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers::contract::EthAbiType)]
    pub enum CkbLightClientCalls {
        Rollback(RollbackCall),
        SetState(SetStateCall),
        Update(UpdateCall),
    }
    impl ethers::core::abi::AbiDecode for CkbLightClientCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <RollbackCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(CkbLightClientCalls::Rollback(decoded));
            }
            if let Ok(decoded) =
                <SetStateCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(CkbLightClientCalls::SetState(decoded));
            }
            if let Ok(decoded) = <UpdateCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(CkbLightClientCalls::Update(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for CkbLightClientCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                CkbLightClientCalls::Rollback(element) => element.encode(),
                CkbLightClientCalls::SetState(element) => element.encode(),
                CkbLightClientCalls::Update(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for CkbLightClientCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                CkbLightClientCalls::Rollback(element) => element.fmt(f),
                CkbLightClientCalls::SetState(element) => element.fmt(f),
                CkbLightClientCalls::Update(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<RollbackCall> for CkbLightClientCalls {
        fn from(var: RollbackCall) -> Self {
            CkbLightClientCalls::Rollback(var)
        }
    }
    impl ::std::convert::From<SetStateCall> for CkbLightClientCalls {
        fn from(var: SetStateCall) -> Self {
            CkbLightClientCalls::SetState(var)
        }
    }
    impl ::std::convert::From<UpdateCall> for CkbLightClientCalls {
        fn from(var: UpdateCall) -> Self {
            CkbLightClientCalls::Update(var)
        }
    }
    #[doc = "`Header(uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,bytes32,bytes32,bytes32,uint128,bytes32)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers::contract::EthAbiType,
        ethers::contract::EthAbiCodec,
    )]
    pub struct Header {
        pub version:           u32,
        pub compact_target:    u32,
        pub timestamp:         u64,
        pub number:            u64,
        pub epoch:             u64,
        pub parent_hash:       [u8; 32],
        pub transactions_root: [u8; 32],
        pub proposals_hash:    [u8; 32],
        pub uncles_hash:       [u8; 32],
        pub dao:               [u8; 32],
        pub nonce:             u128,
        pub block_hash:        [u8; 32],
    }
}
