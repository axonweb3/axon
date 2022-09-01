pub use metadata_contract::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod metadata_contract {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    use ethers_contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers_core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers_providers::Middleware;
    #[doc = "MetadataContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static METADATACONTRACT_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
        ethers_contract::Lazy::new(|| {
            ethers_core::utils::__serde_json::from_str ("[\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"start\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"end\",\n                \"type\": \"uint64\"\n              }\n            ],\n            \"internalType\": \"struct MetadataManager.MetadataVersion\",\n            \"name\": \"version\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"epoch\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_price\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"interval\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"bls_pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"address\",\n                \"name\": \"address_\",\n                \"type\": \"address\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"propose_weight\",\n                \"type\": \"uint32\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"vote_weight\",\n                \"type\": \"uint32\"\n              }\n            ],\n            \"internalType\": \"struct MetadataManager.ValidatorExtend[]\",\n            \"name\": \"verifier_list\",\n            \"type\": \"tuple[]\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"propose_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"prevote_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"precommit_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"brake_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"tx_num_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"max_tx_size\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"last_checkpoint_block_hash\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"internalType\": \"struct MetadataManager.Metadata\",\n        \"name\": \"metadata\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"appendMetadata\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"construct\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"uint64\",\n        \"name\": \"epoch\",\n        \"type\": \"uint64\"\n      }\n    ],\n    \"name\": \"getMetadata\",\n    \"outputs\": [\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"start\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"end\",\n                \"type\": \"uint64\"\n              }\n            ],\n            \"internalType\": \"struct MetadataManager.MetadataVersion\",\n            \"name\": \"version\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"epoch\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_price\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"interval\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"bls_pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"address\",\n                \"name\": \"address_\",\n                \"type\": \"address\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"propose_weight\",\n                \"type\": \"uint32\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"vote_weight\",\n                \"type\": \"uint32\"\n              }\n            ],\n            \"internalType\": \"struct MetadataManager.ValidatorExtend[]\",\n            \"name\": \"verifier_list\",\n            \"type\": \"tuple[]\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"propose_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"prevote_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"precommit_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"brake_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"tx_num_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"max_tx_size\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"last_checkpoint_block_hash\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"internalType\": \"struct MetadataManager.Metadata\",\n        \"name\": \"\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"verifier\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"isProposer\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"relayer\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"isVerifier\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"verifierList\",\n    \"outputs\": [\n      {\n        \"internalType\": \"address[]\",\n        \"name\": \"\",\n        \"type\": \"address[]\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]\n") . expect ("invalid abi")
        });
    pub struct MetadataContract<M>(ethers_contract::Contract<M>);
    impl<M> Clone for MetadataContract<M> {
        fn clone(&self) -> Self {
            MetadataContract(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for MetadataContract<M> {
        type Target = ethers_contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: ethers_providers::Middleware> std::fmt::Debug for MetadataContract<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(MetadataContract))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers_providers::Middleware> MetadataContract<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers_core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers_contract::Contract::new(address.into(), METADATACONTRACT_ABI.clone(), client)
                .into()
        }

        #[doc = "Calls the contract's `appendMetadata` (0x4290d10c) function"]
        pub fn append_metadata(
            &self,
            metadata: Metadata,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([66, 144, 209, 12], (metadata,))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `construct` (0x94b91deb) function"]
        pub fn construct(&self) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([148, 185, 29, 235], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getMetadata` (0x998e84a3) function"]
        pub fn get_metadata(
            &self,
            epoch: u64,
        ) -> ethers_contract::builders::ContractCall<M, Metadata> {
            self.0
                .method_hash([153, 142, 132, 163], epoch)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `isProposer` (0x74ec29a0) function"]
        pub fn is_proposer(
            &self,
            verifier: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([116, 236, 41, 160], verifier)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `isVerifier` (0x33105218) function"]
        pub fn is_verifier(
            &self,
            relayer: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([51, 16, 82, 24], relayer)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `verifierList` (0x870aac5c) function"]
        pub fn verifier_list(
            &self,
        ) -> ethers_contract::builders::ContractCall<
            M,
            (
                ::std::vec::Vec<ethers_core::types::Address>,
                ethers_core::types::U256,
            ),
        > {
            self.0
                .method_hash([135, 10, 172, 92], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers_providers::Middleware> From<ethers_contract::Contract<M>> for MetadataContract<M> {
        fn from(contract: ethers_contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `appendMetadata` function with signature `appendMetadata(((uint64,uint64),uint64,uint64,uint64,uint64,(bytes,bytes,address,uint32,uint32)[],uint64,uint64,uint64,uint64,uint64,uint64,bytes32))` and selector `[66, 144, 209, 12]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(
        name = "appendMetadata",
        abi = "appendMetadata(((uint64,uint64),uint64,uint64,uint64,uint64,(bytes,bytes,address,uint32,uint32)[],uint64,uint64,uint64,uint64,uint64,uint64,bytes32))"
    )]
    pub struct AppendMetadataCall {
        pub metadata: Metadata,
    }
    #[doc = "Container type for all input parameters for the `construct` function with signature `construct()` and selector `[148, 185, 29, 235]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "construct", abi = "construct()")]
    pub struct ConstructCall;
    #[doc = "Container type for all input parameters for the `getMetadata` function with signature `getMetadata(uint64)` and selector `[153, 142, 132, 163]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "getMetadata", abi = "getMetadata(uint64)")]
    pub struct GetMetadataCall {
        pub epoch: u64,
    }
    #[doc = "Container type for all input parameters for the `isProposer` function with signature `isProposer(address)` and selector `[116, 236, 41, 160]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "isProposer", abi = "isProposer(address)")]
    pub struct IsProposerCall {
        pub verifier: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `isVerifier` function with signature `isVerifier(address)` and selector `[51, 16, 82, 24]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "isVerifier", abi = "isVerifier(address)")]
    pub struct IsVerifierCall {
        pub relayer: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `verifierList` function with signature `verifierList()` and selector `[135, 10, 172, 92]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "verifierList", abi = "verifierList()")]
    pub struct VerifierListCall;
    #[derive(Debug, Clone, PartialEq, Eq, ethers_contract::EthAbiType)]
    pub enum MetadataContractCalls {
        AppendMetadata(AppendMetadataCall),
        Construct(ConstructCall),
        GetMetadata(GetMetadataCall),
        IsProposer(IsProposerCall),
        IsVerifier(IsVerifierCall),
        VerifierList(VerifierListCall),
    }
    impl ethers_core::abi::AbiDecode for MetadataContractCalls {
        fn decode(data: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
            if let Ok(decoded) =
                <AppendMetadataCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MetadataContractCalls::AppendMetadata(decoded));
            }
            if let Ok(decoded) =
                <ConstructCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MetadataContractCalls::Construct(decoded));
            }
            if let Ok(decoded) =
                <GetMetadataCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MetadataContractCalls::GetMetadata(decoded));
            }
            if let Ok(decoded) =
                <IsProposerCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MetadataContractCalls::IsProposer(decoded));
            }
            if let Ok(decoded) =
                <IsVerifierCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MetadataContractCalls::IsVerifier(decoded));
            }
            if let Ok(decoded) =
                <VerifierListCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MetadataContractCalls::VerifierList(decoded));
            }
            Err(ethers_core::abi::Error::InvalidData.into())
        }
    }
    impl ethers_core::abi::AbiEncode for MetadataContractCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                MetadataContractCalls::AppendMetadata(element) => element.encode(),
                MetadataContractCalls::Construct(element) => element.encode(),
                MetadataContractCalls::GetMetadata(element) => element.encode(),
                MetadataContractCalls::IsProposer(element) => element.encode(),
                MetadataContractCalls::IsVerifier(element) => element.encode(),
                MetadataContractCalls::VerifierList(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for MetadataContractCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                MetadataContractCalls::AppendMetadata(element) => element.fmt(f),
                MetadataContractCalls::Construct(element) => element.fmt(f),
                MetadataContractCalls::GetMetadata(element) => element.fmt(f),
                MetadataContractCalls::IsProposer(element) => element.fmt(f),
                MetadataContractCalls::IsVerifier(element) => element.fmt(f),
                MetadataContractCalls::VerifierList(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AppendMetadataCall> for MetadataContractCalls {
        fn from(var: AppendMetadataCall) -> Self {
            MetadataContractCalls::AppendMetadata(var)
        }
    }
    impl ::std::convert::From<ConstructCall> for MetadataContractCalls {
        fn from(var: ConstructCall) -> Self {
            MetadataContractCalls::Construct(var)
        }
    }
    impl ::std::convert::From<GetMetadataCall> for MetadataContractCalls {
        fn from(var: GetMetadataCall) -> Self {
            MetadataContractCalls::GetMetadata(var)
        }
    }
    impl ::std::convert::From<IsProposerCall> for MetadataContractCalls {
        fn from(var: IsProposerCall) -> Self {
            MetadataContractCalls::IsProposer(var)
        }
    }
    impl ::std::convert::From<IsVerifierCall> for MetadataContractCalls {
        fn from(var: IsVerifierCall) -> Self {
            MetadataContractCalls::IsVerifier(var)
        }
    }
    impl ::std::convert::From<VerifierListCall> for MetadataContractCalls {
        fn from(var: VerifierListCall) -> Self {
            MetadataContractCalls::VerifierList(var)
        }
    }
    #[doc = "Container type for all return fields from the `getMetadata` function with signature `getMetadata(uint64)` and selector `[153, 142, 132, 163]`"]
    pub struct GetMetadataReturn(
        pub  (
            (u64, u64),
            u64,
            u64,
            u64,
            u64,
            Vec<(
                ethers_core::types::Bytes,
                ethers_core::types::Bytes,
                ethers_core::types::Address,
                u32,
                u32,
            )>,
            u64,
            u64,
            u64,
            u64,
            u64,
            u64,
            [u8; 32],
        ),
    );
    #[doc = "Container type for all return fields from the `isProposer` function with signature `isProposer(address)` and selector `[116, 236, 41, 160]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
    )]
    pub struct IsProposerReturn(pub bool);
    #[doc = "Container type for all return fields from the `isVerifier` function with signature `isVerifier(address)` and selector `[51, 16, 82, 24]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
    )]
    pub struct IsVerifierReturn(pub bool);
    #[doc = "Container type for all return fields from the `verifierList` function with signature `verifierList()` and selector `[135, 10, 172, 92]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
    )]
    pub struct VerifierListReturn(
        pub ::std::vec::Vec<ethers_core::types::Address>,
        pub ethers_core::types::U256,
    );
    #[doc = "`Metadata((uint64,uint64),uint64,uint64,uint64,uint64,(bytes,bytes,address,uint32,uint32)[],uint64,uint64,uint64,uint64,uint64,uint64,bytes32)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
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
    #[doc = "`MetadataVersion(uint64,uint64)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
    )]
    pub struct MetadataVersion {
        pub start: u64,
        pub end:   u64,
    }
    #[doc = "`ValidatorExtend(bytes,bytes,address,uint32,uint32)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
    )]
    pub struct ValidatorExtend {
        pub bls_pub_key:    ethers_core::types::Bytes,
        pub pub_key:        ethers_core::types::Bytes,
        pub address:        ethers_core::types::Address,
        pub propose_weight: u32,
        pub vote_weight:    u32,
    }
}
