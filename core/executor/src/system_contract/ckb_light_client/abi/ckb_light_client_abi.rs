pub use ckb_light_client_contract::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types
)]
pub mod ckb_light_client_contract {
    #[rustfmt::skip]
    const __ABI: &str = "[\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes32[]\",\n        \"name\": \"blockHashes\",\n        \"type\": \"bytes32[]\"\n      }\n    ],\n    \"name\": \"rollback\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"allowRead\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"name\": \"setState\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"uint32\",\n            \"name\": \"version\",\n            \"type\": \"uint32\"\n          },\n          {\n            \"internalType\": \"uint32\",\n            \"name\": \"compactTarget\",\n            \"type\": \"uint32\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"timestamp\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"number\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"epoch\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"parentHash\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"transactionsRoot\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"proposalsHash\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"extraHash\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"dao\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"uint128\",\n            \"name\": \"nonce\",\n            \"type\": \"uint128\"\n          },\n          {\n            \"internalType\": \"bytes\",\n            \"name\": \"extension\",\n            \"type\": \"bytes\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"blockHash\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"internalType\": \"struct CkbType.Header[]\",\n        \"name\": \"headers\",\n        \"type\": \"tuple[]\"\n      }\n    ],\n    \"name\": \"update\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]\n";
    /// The parsed JSON ABI of the contract.
    pub static CKBLIGHTCLIENTCONTRACT_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(|| {
            ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid")
        });
    pub struct CkbLightClientContract<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for CkbLightClientContract<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for CkbLightClientContract<M> {
        type Target = ::ethers::contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for CkbLightClientContract<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for CkbLightClientContract<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(CkbLightClientContract))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> CkbLightClientContract<M> {
        /// Creates a new contract instance with the specified `ethers` client
        /// at `address`. The contract derefs to a `ethers::Contract`
        /// object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                CKBLIGHTCLIENTCONTRACT_ABI.clone(),
                client,
            ))
        }

        /// Calls the contract's `rollback` (0xd32a2285) function
        pub fn rollback(
            &self,
            block_hashes: ::std::vec::Vec<[u8; 32]>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([211, 42, 34, 133], block_hashes)
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `setState` (0xac9f0222) function
        pub fn set_state(
            &self,
            allow_read: bool,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([172, 159, 2, 34], allow_read)
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `update` (0x5d4e8442) function
        pub fn update(
            &self,
            headers: ::std::vec::Vec<Header>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([93, 78, 132, 66], headers)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for CkbLightClientContract<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    /// Container type for all input parameters for the `rollback` function with
    /// signature `rollback(bytes32[])` and selector `0xd32a2285`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "rollback", abi = "rollback(bytes32[])")]
    pub struct RollbackCall {
        pub block_hashes: ::std::vec::Vec<[u8; 32]>,
    }
    /// Container type for all input parameters for the `setState` function with
    /// signature `setState(bool)` and selector `0xac9f0222`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "setState", abi = "setState(bool)")]
    pub struct SetStateCall {
        pub allow_read: bool,
    }
    /// Container type for all input parameters for the `update` function with
    /// signature `update((uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,
    /// bytes32,bytes32,bytes32,uint128,bytes,bytes32)[])` and selector
    /// `0x5d4e8442`
    #[derive(Clone, ::ethers::contract::EthCall, ::ethers::contract::EthDisplay)]
    #[ethcall(
        name = "update",
        abi = "update((uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,bytes32,bytes32,bytes32,uint128,bytes,bytes32)[])"
    )]
    pub struct UpdateCall {
        pub headers: ::std::vec::Vec<Header>,
    }
    /// Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType)]
    pub enum CkbLightClientContractCalls {
        Rollback(RollbackCall),
        SetState(SetStateCall),
        Update(UpdateCall),
    }
    impl ::ethers::core::abi::AbiDecode for CkbLightClientContractCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <RollbackCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Rollback(decoded));
            }
            if let Ok(decoded) = <SetStateCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SetState(decoded));
            }
            if let Ok(decoded) = <UpdateCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Update(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for CkbLightClientContractCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Rollback(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SetState(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Update(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for CkbLightClientContractCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Rollback(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetState(element) => ::core::fmt::Display::fmt(element, f),
                Self::Update(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<RollbackCall> for CkbLightClientContractCalls {
        fn from(value: RollbackCall) -> Self {
            Self::Rollback(value)
        }
    }
    impl ::core::convert::From<SetStateCall> for CkbLightClientContractCalls {
        fn from(value: SetStateCall) -> Self {
            Self::SetState(value)
        }
    }
    impl ::core::convert::From<UpdateCall> for CkbLightClientContractCalls {
        fn from(value: UpdateCall) -> Self {
            Self::Update(value)
        }
    }
    /// `Header(uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,bytes32,
    /// bytes32,bytes32,uint128,bytes,bytes32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
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
        pub extra_hash:        [u8; 32],
        pub dao:               [u8; 32],
        pub nonce:             u128,
        pub extension:         ::ethers::core::types::Bytes,
        pub block_hash:        [u8; 32],
    }
}
