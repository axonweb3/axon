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
    const __ABI: &str = "[\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"start\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"end\",\n                \"type\": \"uint64\"\n              }\n            ],\n            \"internalType\": \"struct MetadataType.MetadataVersion\",\n            \"name\": \"version\",\n            \"type\": \"tuple\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"epoch\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"bls_pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"pub_key\",\n                \"type\": \"bytes\"\n              },\n              {\n                \"internalType\": \"address\",\n                \"name\": \"address_\",\n                \"type\": \"address\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"propose_weight\",\n                \"type\": \"uint32\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"vote_weight\",\n                \"type\": \"uint32\"\n              }\n            ],\n            \"internalType\": \"struct MetadataType.ValidatorExtend[]\",\n            \"name\": \"verifier_list\",\n            \"type\": \"tuple[]\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"address\",\n                \"name\": \"address_\",\n                \"type\": \"address\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"count\",\n                \"type\": \"uint64\"\n              }\n            ],\n            \"internalType\": \"struct MetadataType.ProposeCount[]\",\n            \"name\": \"propose_counter\",\n            \"type\": \"tuple[]\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"propose_ratio\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"prevote_ratio\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"precommit_ratio\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"brake_ratio\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"tx_num_limit\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"max_tx_size\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"gas_limit\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"interval\",\n                \"type\": \"uint64\"\n              },\n              {\n                \"internalType\": \"uint64\",\n                \"name\": \"max_contract_limit\",\n                \"type\": \"uint64\"\n              }\n            ],\n            \"internalType\": \"struct MetadataType.ConsensusConfig\",\n            \"name\": \"consensus_config\",\n            \"type\": \"tuple\"\n          }\n        ],\n        \"internalType\": \"struct MetadataType.Metadata\",\n        \"name\": \"metadata\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"appendMetadata\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"metadata_type_id\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"checkpoint_type_id\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"xudt_args\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"stake_smt_type_id\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"delegate_smt_type_id\",\n            \"type\": \"bytes32\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"reward_smt_type_id\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"internalType\": \"struct MetadataType.CkbRelatedInfo\",\n        \"name\": \"info\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"setCkbRelatedInfo\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"propose_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"prevote_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"precommit_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"brake_ratio\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"tx_num_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"max_tx_size\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"gas_limit\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"interval\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"max_contract_limit\",\n            \"type\": \"uint64\"\n          }\n        ],\n        \"internalType\": \"struct MetadataType.ConsensusConfig\",\n        \"name\": \"config\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"updateConsensusConfig\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  }\n]\n";
    /// The parsed JSON ABI of the contract.
    pub static METADATACONTRACT_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(
            || ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid")
        );
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

        /// Calls the contract's `appendMetadata` (0x53ec79e6) function
        pub fn append_metadata(
            &self,
            metadata: Metadata,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([83, 236, 121, 230], (metadata,))
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `setCkbRelatedInfo` (0x804afc59) function
        pub fn set_ckb_related_info(
            &self,
            info: CkbRelatedInfo,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([128, 74, 252, 89], (info,))
                .expect("method not found (this should never happen)")
        }

        /// Calls the contract's `updateConsensusConfig` (0xb76fac01) function
        pub fn update_consensus_config(
            &self,
            config: ConsensusConfig,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([183, 111, 172, 1], (config,))
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
    /// `appendMetadata(((uint64,uint64),uint64,(bytes,bytes,address,uint32,
    /// uint32)[],(address,uint64)[],(uint64,uint64,uint64,uint64,uint64,uint64,
    /// uint64,uint64,uint64)))` and selector `0x53ec79e6`
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
    #[ethcall(
        name = "appendMetadata",
        abi = "appendMetadata(((uint64,uint64),uint64,(bytes,bytes,address,uint32,uint32)[],(address,uint64)[],(uint64,uint64,uint64,uint64,uint64,uint64,uint64,uint64,uint64)))"
    )]
    pub struct AppendMetadataCall {
        pub metadata: Metadata,
    }
    /// Container type for all input parameters for the `setCkbRelatedInfo`
    /// function with signature
    /// `setCkbRelatedInfo((bytes32,bytes32,bytes32,bytes32,bytes32,bytes32))`
    /// and selector `0x804afc59`
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
    #[ethcall(
        name = "setCkbRelatedInfo",
        abi = "setCkbRelatedInfo((bytes32,bytes32,bytes32,bytes32,bytes32,bytes32))"
    )]
    pub struct SetCkbRelatedInfoCall {
        pub info: CkbRelatedInfo,
    }
    /// Container type for all input parameters for the `updateConsensusConfig`
    /// function with signature
    /// `updateConsensusConfig((uint64,uint64,uint64,uint64,uint64,uint64,
    /// uint64,uint64,uint64))` and selector `0xb76fac01`
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
    #[ethcall(
        name = "updateConsensusConfig",
        abi = "updateConsensusConfig((uint64,uint64,uint64,uint64,uint64,uint64,uint64,uint64,uint64))"
    )]
    pub struct UpdateConsensusConfigCall {
        pub config: ConsensusConfig,
    }
    /// Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum MetadataContractCalls {
        AppendMetadata(AppendMetadataCall),
        SetCkbRelatedInfo(SetCkbRelatedInfoCall),
        UpdateConsensusConfig(UpdateConsensusConfigCall),
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
            if let Ok(decoded) =
                <SetCkbRelatedInfoCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SetCkbRelatedInfo(decoded));
            }
            if let Ok(decoded) =
                <UpdateConsensusConfigCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::UpdateConsensusConfig(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for MetadataContractCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AppendMetadata(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SetCkbRelatedInfo(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::UpdateConsensusConfig(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for MetadataContractCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AppendMetadata(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetCkbRelatedInfo(element) => ::core::fmt::Display::fmt(element, f),
                Self::UpdateConsensusConfig(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AppendMetadataCall> for MetadataContractCalls {
        fn from(value: AppendMetadataCall) -> Self {
            Self::AppendMetadata(value)
        }
    }
    impl ::core::convert::From<SetCkbRelatedInfoCall> for MetadataContractCalls {
        fn from(value: SetCkbRelatedInfoCall) -> Self {
            Self::SetCkbRelatedInfo(value)
        }
    }
    impl ::core::convert::From<UpdateConsensusConfigCall> for MetadataContractCalls {
        fn from(value: UpdateConsensusConfigCall) -> Self {
            Self::UpdateConsensusConfig(value)
        }
    }
    /// `CkbRelatedInfo(bytes32,bytes32,bytes32,bytes32,bytes32,bytes32)`
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
    pub struct CkbRelatedInfo {
        pub metadata_type_id:     [u8; 32],
        pub checkpoint_type_id:   [u8; 32],
        pub xudt_args:            [u8; 32],
        pub stake_smt_type_id:    [u8; 32],
        pub delegate_smt_type_id: [u8; 32],
        pub reward_smt_type_id:   [u8; 32],
    }
    /// `ConsensusConfig(uint64,uint64,uint64,uint64,uint64,uint64,uint64,
    /// uint64,uint64)`
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
    pub struct ConsensusConfig {
        pub propose_ratio:      u64,
        pub prevote_ratio:      u64,
        pub precommit_ratio:    u64,
        pub brake_ratio:        u64,
        pub tx_num_limit:       u64,
        pub max_tx_size:        u64,
        pub gas_limit:          u64,
        pub interval:           u64,
        pub max_contract_limit: u64,
    }
    /// `Metadata((uint64,uint64),uint64,(bytes,bytes,address,uint32,uint32)[],
    /// (address,uint64)[],(uint64,uint64,uint64,uint64,uint64,uint64,uint64,
    /// uint64,uint64))`
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
        pub version:          MetadataVersion,
        pub epoch:            u64,
        pub verifier_list:    ::std::vec::Vec<ValidatorExtend>,
        pub propose_counter:  ::std::vec::Vec<ProposeCount>,
        pub consensus_config: ConsensusConfig,
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
    /// `ProposeCount(address,uint64)`
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
    pub struct ProposeCount {
        pub address: ::ethers::core::types::Address,
        pub count:   u64,
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
