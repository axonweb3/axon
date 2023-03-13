pub use image_cell_contract::*;
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
pub mod image_cell_contract {
    #[rustfmt::skip]
    const __ABI: &str = "[\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"components\": [\n              {\n                \"internalType\": \"bytes32\",\n                \"name\": \"txHash\",\n                \"type\": \"bytes32\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"index\",\n                \"type\": \"uint32\"\n              }\n            ],\n            \"internalType\": \"struct CkbType.OutPoint[]\",\n            \"name\": \"txInputs\",\n            \"type\": \"tuple[]\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"bytes32\",\n                \"name\": \"txHash\",\n                \"type\": \"bytes32\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"index\",\n                \"type\": \"uint32\"\n              }\n            ],\n            \"internalType\": \"struct CkbType.OutPoint[]\",\n            \"name\": \"txOutputs\",\n            \"type\": \"tuple[]\"\n          }\n        ],\n        \"internalType\": \"struct ImageCell.BlockRollBlack[]\",\n        \"name\": \"blocks\",\n        \"type\": \"tuple[]\"\n      }\n    ],\n    \"name\": \"rollback\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"allowRead\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"name\": \"setState\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"uint64\",\n            \"name\": \"blockNumber\",\n            \"type\": \"uint64\"\n          },\n          {\n            \"components\": [\n              {\n                \"internalType\": \"bytes32\",\n                \"name\": \"txHash\",\n                \"type\": \"bytes32\"\n              },\n              {\n                \"internalType\": \"uint32\",\n                \"name\": \"index\",\n                \"type\": \"uint32\"\n              }\n            ],\n            \"internalType\": \"struct CkbType.OutPoint[]\",\n            \"name\": \"txInputs\",\n            \"type\": \"tuple[]\"\n          },\n          {\n            \"components\": [\n              {\n                \"components\": [\n                  {\n                    \"internalType\": \"bytes32\",\n                    \"name\": \"txHash\",\n                    \"type\": \"bytes32\"\n                  },\n                  {\n                    \"internalType\": \"uint32\",\n                    \"name\": \"index\",\n                    \"type\": \"uint32\"\n                  }\n                ],\n                \"internalType\": \"struct CkbType.OutPoint\",\n                \"name\": \"outPoint\",\n                \"type\": \"tuple\"\n              },\n              {\n                \"components\": [\n                  {\n                    \"internalType\": \"uint64\",\n                    \"name\": \"capacity\",\n                    \"type\": \"uint64\"\n                  },\n                  {\n                    \"components\": [\n                      {\n                        \"internalType\": \"bytes32\",\n                        \"name\": \"codeHash\",\n                        \"type\": \"bytes32\"\n                      },\n                      {\n                        \"internalType\": \"enum CkbType.ScriptHashType\",\n                        \"name\": \"hashType\",\n                        \"type\": \"uint8\"\n                      },\n                      {\n                        \"internalType\": \"bytes\",\n                        \"name\": \"args\",\n                        \"type\": \"bytes\"\n                      }\n                    ],\n                    \"internalType\": \"struct CkbType.Script\",\n                    \"name\": \"lock\",\n                    \"type\": \"tuple\"\n                  },\n                  {\n                    \"components\": [\n                      {\n                        \"internalType\": \"bytes32\",\n                        \"name\": \"codeHash\",\n                        \"type\": \"bytes32\"\n                      },\n                      {\n                        \"internalType\": \"enum CkbType.ScriptHashType\",\n                        \"name\": \"hashType\",\n                        \"type\": \"uint8\"\n                      },\n                      {\n                        \"internalType\": \"bytes\",\n                        \"name\": \"args\",\n                        \"type\": \"bytes\"\n                      }\n                    ],\n                    \"internalType\": \"struct CkbType.Script[]\",\n                    \"name\": \"type_\",\n                    \"type\": \"tuple[]\"\n                  }\n                ],\n                \"internalType\": \"struct CkbType.CellOutput\",\n                \"name\": \"output\",\n                \"type\": \"tuple\"\n              },\n              {\n                \"internalType\": \"bytes\",\n                \"name\": \"data\",\n                \"type\": \"bytes\"\n              }\n            ],\n            \"internalType\": \"struct CkbType.CellInfo[]\",\n            \"name\": \"txOutputs\",\n            \"type\": \"tuple[]\"\n          }\n        ],\n        \"internalType\": \"struct ImageCell.BlockUpdate[]\",\n        \"name\": \"blocks\",\n        \"type\": \"tuple[]\"\n      }\n    ],\n    \"name\": \"update\",\n    \"outputs\": [],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]\n";
    /// The parsed JSON ABI of the contract.
    pub static IMAGECELLCONTRACT_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(|| {
            ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid")
        });
    pub struct ImageCellContract<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for ImageCellContract<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for ImageCellContract<M> {
        type Target = ::ethers::contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for ImageCellContract<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for ImageCellContract<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(ImageCellContract))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> ImageCellContract<M> {
        /// Creates a new contract instance with the specified `ethers` client
        /// at `address`. The contract derefs to a `ethers::Contract`
        /// object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                IMAGECELLCONTRACT_ABI.clone(),
                client,
            ))
        }

        /// Calls the contract's `rollback` (0x08c17228) function
        pub fn rollback(
            &self,
            blocks: ::std::vec::Vec<BlockRollBlack>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([8, 193, 114, 40], blocks)
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

        /// Calls the contract's `update` (0xafa74e04) function
        pub fn update(
            &self,
            blocks: ::std::vec::Vec<BlockUpdate>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([175, 167, 78, 4], blocks)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for ImageCellContract<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    /// Container type for all input parameters for the `rollback` function with
    /// signature `rollback(((bytes32,uint32)[],(bytes32,uint32)[])[])` and
    /// selector `0x08c17228`
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
        name = "rollback",
        abi = "rollback(((bytes32,uint32)[],(bytes32,uint32)[])[])"
    )]
    pub struct RollbackCall {
        pub blocks: ::std::vec::Vec<BlockRollBlack>,
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
    /// signature `update((uint64,(bytes32,uint32)[],((bytes32,uint32),(uint64,
    /// (bytes32,uint8,bytes),(bytes32,uint8,bytes)[]),bytes)[])[])` and
    /// selector `0xafa74e04`
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
        name = "update",
        abi = "update((uint64,(bytes32,uint32)[],((bytes32,uint32),(uint64,(bytes32,uint8,bytes),(bytes32,uint8,bytes)[]),bytes)[])[])"
    )]
    pub struct UpdateCall {
        pub blocks: ::std::vec::Vec<BlockUpdate>,
    }
    /// Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum ImageCellContractCalls {
        Rollback(RollbackCall),
        SetState(SetStateCall),
        Update(UpdateCall),
    }
    impl ::ethers::core::abi::AbiDecode for ImageCellContractCalls {
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
    impl ::ethers::core::abi::AbiEncode for ImageCellContractCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Rollback(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SetState(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Update(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for ImageCellContractCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Rollback(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetState(element) => ::core::fmt::Display::fmt(element, f),
                Self::Update(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<RollbackCall> for ImageCellContractCalls {
        fn from(value: RollbackCall) -> Self {
            Self::Rollback(value)
        }
    }
    impl ::core::convert::From<SetStateCall> for ImageCellContractCalls {
        fn from(value: SetStateCall) -> Self {
            Self::SetState(value)
        }
    }
    impl ::core::convert::From<UpdateCall> for ImageCellContractCalls {
        fn from(value: UpdateCall) -> Self {
            Self::Update(value)
        }
    }
    /// `CellInfo((bytes32,uint32),(uint64,(bytes32,uint8,bytes),(bytes32,uint8,
    /// bytes)[]),bytes)`
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
    pub struct CellInfo {
        pub out_point: OutPoint,
        pub output:    CellOutput,
        pub data:      ::ethers::core::types::Bytes,
    }
    /// `CellOutput(uint64,(bytes32,uint8,bytes),(bytes32,uint8,bytes)[])`
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
    pub struct CellOutput {
        pub capacity: u64,
        pub lock:     Script,
        pub type_:    ::std::vec::Vec<Script>,
    }
    /// `OutPoint(bytes32,uint32)`
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
    pub struct OutPoint {
        pub tx_hash: [u8; 32],
        pub index:   u32,
    }
    /// `Script(bytes32,uint8,bytes)`
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
    pub struct Script {
        pub code_hash: [u8; 32],
        pub hash_type: u8,
        pub args:      ::ethers::core::types::Bytes,
    }
    /// `BlockRollBlack((bytes32,uint32)[],(bytes32,uint32)[])`
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
    pub struct BlockRollBlack {
        pub tx_inputs:  ::std::vec::Vec<OutPoint>,
        pub tx_outputs: ::std::vec::Vec<OutPoint>,
    }
    /// `BlockUpdate(uint64,(bytes32,uint32)[],((bytes32,uint32),(uint64,
    /// (bytes32,uint8,bytes),(bytes32,uint8,bytes)[]),bytes)[])`
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
    pub struct BlockUpdate {
        pub block_number: u64,
        pub tx_inputs:    ::std::vec::Vec<OutPoint>,
        pub tx_outputs:   ::std::vec::Vec<CellInfo>,
    }
}
