pub use metadata_contract::*;
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
pub mod metadata_contract {
    #[rustfmt::skip]
    const __ABI: &str = "[\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"start\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"end\",\n                \"type\": \"uint64\"\n              }\n            ],\n            \"internalType\": \"struct MetadataManager.MetadataVersion\",\n            \"name\": \"version\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"epoch\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_price\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"interval\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"bls_pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"address\",\n                \"name\": \"address_\",\n                \"type\": \"address\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"propose_weight\",\n                \"type\": \"uint32\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"vote_weight\",\n                \"type\": \"uint32\"\n              }\n            ],\n            \"internalType\": \"struct MetadataManager.ValidatorExtend[]\",\n            \"name\": \"verifier_list\",\n            \"type\": \"tuple[]\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"propose_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"prevote_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"precommit_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"brake_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"tx_num_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"max_tx_size\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"last_checkpoint_block_hash\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"internalType\": \"struct MetadataManager.Metadata\",\n        \"name\": \"metadata\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"appendMetadata\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"construct\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"uint64\",\n        \"name\": \"epoch\",\n        \"type\": \"uint64\"\n      }\n    ],\n    \"name\": \"getMetadata\",\n    \"outputs\": [\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"start\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"end\",\n                \"type\": \"uint64\"\n              }\n            ],\n            \"internalType\": \"struct MetadataManager.MetadataVersion\",\n            \"name\": \"version\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"epoch\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_price\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"interval\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"bls_pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"address\",\n                \"name\": \"address_\",\n                \"type\": \"address\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"propose_weight\",\n                \"type\": \"uint32\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"vote_weight\",\n                \"type\": \"uint32\"\n              }\n            ],\n            \"internalType\": \"struct MetadataManager.ValidatorExtend[]\",\n            \"name\": \"verifier_list\",\n            \"type\": \"tuple[]\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"propose_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"prevote_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"precommit_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"brake_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"tx_num_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"max_tx_size\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"last_checkpoint_block_hash\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"internalType\": \"struct MetadataManager.Metadata\",\n        \"name\": \"\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"verifier\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"isProposer\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"relayer\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"isVerifier\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"verifierList\",\n    \"outputs\": [\n      {\n        \"internalType\": \"address[]\",\n        \"name\": \"\",\n        \"type\": \"address[]\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]\n";
    /// The parsed JSON ABI of the contract.
    pub static METADATACONTRACT_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(|| {
            ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid")
        });
    pub struct MetadataContract<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for MetadataContract<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for MetadataContract<M> {
        type Target = ::ethers::contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for MetadataContract<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for MetadataContract<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(MetadataContract))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> MetadataContract<M> {
        /// Creates a new contract instance with the specified `ethers` client
        /// at `address`. The contract derefs to a `ethers::Contract`
        /// object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                METADATACONTRACT_ABI.clone(),
                client,
            ))
        }

        /// Calls the contract's `appendMetadata` (0x4290d10c) function
        pub fn append_metadata(
            &self,
            metadata: Metadata,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([66, 144, 209, 12], (metadata,))
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `construct` (0x94b91deb) function
        pub fn construct(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([148, 185, 29, 235], ())
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `getMetadata` (0x998e84a3) function
        pub fn get_metadata(
            &self,
            epoch: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, Metadata> {
            self.0
                .method_hash([153, 142, 132, 163], epoch)
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `isProposer` (0x74ec29a0) function
        pub fn is_proposer(
            &self,
            verifier: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([116, 236, 41, 160], verifier)
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `isVerifier` (0x33105218) function
        pub fn is_verifier(
            &self,
            relayer: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([51, 16, 82, 24], relayer)
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `verifierList` (0x870aac5c) function
        pub fn verifier_list(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                ::std::vec::Vec<::ethers::core::types::Address>,
                ::ethers::core::types::U256,
            ),
        > {
            self.0
                .method_hash([135, 10, 172, 92], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for MetadataContract<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    /// Container type for all input parameters for the `appendMetadata`
    /// function with signature
    /// `appendMetadata(((uint64,uint64),uint64,uint64,uint64,uint64,(bytes,
    /// bytes,address,uint32,uint32)[],uint64,uint64,uint64,uint64,uint64,
    /// uint64,bytes32))` and selector `0x4290d10c`
    #[derive(Clone, ::ethers::contract::EthCall, ::ethers::contract::EthDisplay)]
    #[ethcall(
        name = "appendMetadata",
        abi = "appendMetadata(((uint64,uint64),uint64,uint64,uint64,uint64,(bytes,bytes,address,uint32,uint32)[],uint64,uint64,uint64,uint64,uint64,uint64,bytes32))"
    )]
    pub struct AppendMetadataCall {
        pub metadata: Metadata,
    }
    /// Container type for all input parameters for the `construct` function
    /// with signature `construct()` and selector `0x94b91deb`
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
    #[ethcall(name = "construct", abi = "construct()")]
    pub struct ConstructCall;
    /// Container type for all input parameters for the `getMetadata` function
    /// with signature `getMetadata(uint64)` and selector `0x998e84a3`
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
    #[ethcall(name = "getMetadata", abi = "getMetadata(uint64)")]
    pub struct GetMetadataCall {
        pub epoch: u64,
    }
    /// Container type for all input parameters for the `isProposer` function
    /// with signature `isProposer(address)` and selector `0x74ec29a0`
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
    #[ethcall(name = "isProposer", abi = "isProposer(address)")]
    pub struct IsProposerCall {
        pub verifier: ::ethers::core::types::Address,
    }
    /// Container type for all input parameters for the `isVerifier` function
    /// with signature `isVerifier(address)` and selector `0x33105218`
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
    #[ethcall(name = "isVerifier", abi = "isVerifier(address)")]
    pub struct IsVerifierCall {
        pub relayer: ::ethers::core::types::Address,
    }
    /// Container type for all input parameters for the `verifierList` function
    /// with signature `verifierList()` and selector `0x870aac5c`
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
    #[ethcall(name = "verifierList", abi = "verifierList()")]
    pub struct VerifierListCall;
    /// Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType)]
    pub enum MetadataContractCalls {
        AppendMetadata(AppendMetadataCall),
        Construct(ConstructCall),
        GetMetadata(GetMetadataCall),
        IsProposer(IsProposerCall),
        IsVerifier(IsVerifierCall),
        VerifierList(VerifierListCall),
    }
    impl ::ethers::core::abi::AbiDecode for MetadataContractCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <AppendMetadataCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::AppendMetadata(decoded));
            }
            if let Ok(decoded) = <ConstructCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Construct(decoded));
            }
            if let Ok(decoded) = <GetMetadataCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetMetadata(decoded));
            }
            if let Ok(decoded) = <IsProposerCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::IsProposer(decoded));
            }
            if let Ok(decoded) = <IsVerifierCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::IsVerifier(decoded));
            }
            if let Ok(decoded) = <VerifierListCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::VerifierList(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for MetadataContractCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AppendMetadata(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Construct(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetMetadata(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::IsProposer(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::IsVerifier(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::VerifierList(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for MetadataContractCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AppendMetadata(element) => ::core::fmt::Display::fmt(element, f),
                Self::Construct(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetMetadata(element) => ::core::fmt::Display::fmt(element, f),
                Self::IsProposer(element) => ::core::fmt::Display::fmt(element, f),
                Self::IsVerifier(element) => ::core::fmt::Display::fmt(element, f),
                Self::VerifierList(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AppendMetadataCall> for MetadataContractCalls {
        fn from(value: AppendMetadataCall) -> Self {
            Self::AppendMetadata(value)
        }
    }
    impl ::core::convert::From<ConstructCall> for MetadataContractCalls {
        fn from(value: ConstructCall) -> Self {
            Self::Construct(value)
        }
    }
    impl ::core::convert::From<GetMetadataCall> for MetadataContractCalls {
        fn from(value: GetMetadataCall) -> Self {
            Self::GetMetadata(value)
        }
    }
    impl ::core::convert::From<IsProposerCall> for MetadataContractCalls {
        fn from(value: IsProposerCall) -> Self {
            Self::IsProposer(value)
        }
    }
    impl ::core::convert::From<IsVerifierCall> for MetadataContractCalls {
        fn from(value: IsVerifierCall) -> Self {
            Self::IsVerifier(value)
        }
    }
    impl ::core::convert::From<VerifierListCall> for MetadataContractCalls {
        fn from(value: VerifierListCall) -> Self {
            Self::VerifierList(value)
        }
    }
    /// Container type for all return fields from the `getMetadata` function
    /// with signature `getMetadata(uint64)` and selector `0x998e84a3`
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
    pub struct GetMetadataReturn(pub Metadata);
    /// Container type for all return fields from the `isProposer` function with
    /// signature `isProposer(address)` and selector `0x74ec29a0`
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
    pub struct IsProposerReturn(pub bool);
    /// Container type for all return fields from the `isVerifier` function with
    /// signature `isVerifier(address)` and selector `0x33105218`
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
    pub struct IsVerifierReturn(pub bool);
    /// Container type for all return fields from the `verifierList` function
    /// with signature `verifierList()` and selector `0x870aac5c`
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
    pub struct VerifierListReturn(
        pub ::std::vec::Vec<::ethers::core::types::Address>,
        pub ::ethers::core::types::U256,
    );
    /// `Metadata((uint64,uint64),uint64,uint64,uint64,uint64,(bytes,bytes,
    /// address,uint32,uint32)[],uint64,uint64,uint64,uint64,uint64,uint64,
    /// bytes32)`
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
    pub struct Metadata {
        pub version:                    MetadataVersion,
        pub epoch:                      u64,
        pub gas_limit:                  u64,
        pub gas_price:                  u64,
        pub interval:                   u64,
        pub verifier_list:              ::std::vec::Vec<ValidatorExtend>,
        pub propose_ratio:              u64,
        pub prevote_ratio:              u64,
        pub precommit_ratio:            u64,
        pub brake_ratio:                u64,
        pub tx_num_limit:               u64,
        pub max_tx_size:                u64,
        pub last_checkpoint_block_hash: [u8; 32],
    }
    /// `MetadataVersion(uint64,uint64)`
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
    pub struct MetadataVersion {
        pub start: u64,
        pub end:   u64,
    }
    /// `ValidatorExtend(bytes,bytes,address,uint32,uint32)`
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
    pub struct ValidatorExtend {
        pub bls_pub_key:    ::ethers::core::types::Bytes,
        pub pub_key:        ::ethers::core::types::Bytes,
        pub address:        ::ethers::core::types::Address,
        pub propose_weight: u32,
        pub vote_weight:    u32,
    }
}
