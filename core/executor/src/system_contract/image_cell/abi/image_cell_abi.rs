pub use image_cell::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod image_cell {
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
    #[doc = "ImageCell was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[\n    {\n      \"inputs\": [\n        {\n          \"internalType\": \"bytes32\",\n          \"name\": \"blockHash\",\n          \"type\": \"bytes32\"\n        },\n        {\n          \"internalType\": \"uint64\",\n          \"name\": \"blockNumber\",\n          \"type\": \"uint64\"\n        },\n        {\n          \"components\": [\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"txHash\",\n              \"type\": \"bytes32\"\n            },\n            {\n              \"internalType\": \"uint32\",\n              \"name\": \"index\",\n              \"type\": \"uint32\"\n            }\n          ],\n          \"internalType\": \"struct CkbType.OutPoint[]\",\n          \"name\": \"inputs\",\n          \"type\": \"tuple[]\"\n        },\n        {\n          \"components\": [\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"txHash\",\n              \"type\": \"bytes32\"\n            },\n            {\n              \"internalType\": \"uint32\",\n              \"name\": \"index\",\n              \"type\": \"uint32\"\n            }\n          ],\n          \"internalType\": \"struct CkbType.OutPoint[]\",\n          \"name\": \"outputs\",\n          \"type\": \"tuple[]\"\n        }\n      ],\n      \"name\": \"rollback\",\n      \"outputs\": [],\n      \"stateMutability\": \"view\",\n      \"type\": \"function\"\n    },\n    {\n      \"inputs\": [\n        {\n          \"internalType\": \"bool\",\n          \"name\": \"allowRead\",\n          \"type\": \"bool\"\n        }\n      ],\n      \"name\": \"setState\",\n      \"outputs\": [],\n      \"stateMutability\": \"view\",\n      \"type\": \"function\"\n    },\n    {\n      \"inputs\": [\n        {\n          \"components\": [\n            {\n              \"internalType\": \"uint32\",\n              \"name\": \"version\",\n              \"type\": \"uint32\"\n            },\n            {\n              \"internalType\": \"uint32\",\n              \"name\": \"compactTarget\",\n              \"type\": \"uint32\"\n            },\n            {\n              \"internalType\": \"uint64\",\n              \"name\": \"timestamp\",\n              \"type\": \"uint64\"\n            },\n            {\n              \"internalType\": \"uint64\",\n              \"name\": \"number\",\n              \"type\": \"uint64\"\n            },\n            {\n              \"internalType\": \"uint64\",\n              \"name\": \"epoch\",\n              \"type\": \"uint64\"\n            },\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"parentHash\",\n              \"type\": \"bytes32\"\n            },\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"transactionsRoot\",\n              \"type\": \"bytes32\"\n            },\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"proposalsHash\",\n              \"type\": \"bytes32\"\n            },\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"unclesHash\",\n              \"type\": \"bytes32\"\n            },\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"dao\",\n              \"type\": \"bytes32\"\n            },\n            {\n              \"internalType\": \"uint128\",\n              \"name\": \"nonce\",\n              \"type\": \"uint128\"\n            },\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"blockHash\",\n              \"type\": \"bytes32\"\n            }\n          ],\n          \"internalType\": \"struct CkbType.Header\",\n          \"name\": \"header\",\n          \"type\": \"tuple\"\n        },\n        {\n          \"components\": [\n            {\n              \"internalType\": \"bytes32\",\n              \"name\": \"txHash\",\n              \"type\": \"bytes32\"\n            },\n            {\n              \"internalType\": \"uint32\",\n              \"name\": \"index\",\n              \"type\": \"uint32\"\n            }\n          ],\n          \"internalType\": \"struct CkbType.OutPoint[]\",\n          \"name\": \"inputs\",\n          \"type\": \"tuple[]\"\n        },\n        {\n          \"components\": [\n            {\n              \"components\": [\n                {\n                  \"internalType\": \"bytes32\",\n                  \"name\": \"txHash\",\n                  \"type\": \"bytes32\"\n                },\n                {\n                  \"internalType\": \"uint32\",\n                  \"name\": \"index\",\n                  \"type\": \"uint32\"\n                }\n              ],\n              \"internalType\": \"struct CkbType.OutPoint\",\n              \"name\": \"outPoint\",\n              \"type\": \"tuple\"\n            },\n            {\n              \"components\": [\n                {\n                  \"internalType\": \"uint64\",\n                  \"name\": \"capacity\",\n                  \"type\": \"uint64\"\n                },\n                {\n                  \"components\": [\n                    {\n                      \"internalType\": \"bytes32\",\n                      \"name\": \"codeHash\",\n                      \"type\": \"bytes32\"\n                    },\n                    {\n                      \"internalType\": \"enum CkbType.ScriptHashType\",\n                      \"name\": \"hashType\",\n                      \"type\": \"uint8\"\n                    },\n                    {\n                      \"internalType\": \"bytes\",\n                      \"name\": \"args\",\n                      \"type\": \"bytes\"\n                    }\n                  ],\n                  \"internalType\": \"struct CkbType.Script\",\n                  \"name\": \"lock\",\n                  \"type\": \"tuple\"\n                },\n                {\n                  \"components\": [\n                    {\n                      \"internalType\": \"bytes32\",\n                      \"name\": \"codeHash\",\n                      \"type\": \"bytes32\"\n                    },\n                    {\n                      \"internalType\": \"enum CkbType.ScriptHashType\",\n                      \"name\": \"hashType\",\n                      \"type\": \"uint8\"\n                    },\n                    {\n                      \"internalType\": \"bytes\",\n                      \"name\": \"args\",\n                      \"type\": \"bytes\"\n                    }\n                  ],\n                  \"internalType\": \"struct CkbType.Script[]\",\n                  \"name\": \"type_\",\n                  \"type\": \"tuple[]\"\n                }\n              ],\n              \"internalType\": \"struct CkbType.CellOutput\",\n              \"name\": \"output\",\n              \"type\": \"tuple\"\n            },\n            {\n              \"internalType\": \"bytes\",\n              \"name\": \"data\",\n              \"type\": \"bytes\"\n            }\n          ],\n          \"internalType\": \"struct CkbType.CellInfo[]\",\n          \"name\": \"outputs\",\n          \"type\": \"tuple[]\"\n        }\n      ],\n      \"name\": \"update\",\n      \"outputs\": [],\n      \"stateMutability\": \"view\",\n      \"type\": \"function\"\n    }\n]\n" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static IMAGECELL_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct ImageCell<M>(ethers::contract::Contract<M>);
    impl<M> Clone for ImageCell<M> {
        fn clone(&self) -> Self {
            ImageCell(self.0.clone())
        }
    }

    impl<M> std::ops::Deref for ImageCell<M> {
        type Target = ethers::contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for ImageCell<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(ImageCell))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> ImageCell<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), IMAGECELL_ABI.clone(), client).into()
        }

        #[doc = "Calls the contract's `rollback` (0xe2594d22) function"]
        pub fn rollback(
            &self,
            block_hash: [u8; 32],
            block_number: u64,
            inputs: ::std::vec::Vec<OutPoint>,
            outputs: ::std::vec::Vec<OutPoint>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [226, 89, 77, 34],
                    (block_hash, block_number, inputs, outputs),
                )
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

        #[doc = "Calls the contract's `update` (0x09236448) function"]
        pub fn update(
            &self,
            header: Header,
            inputs: ::std::vec::Vec<OutPoint>,
            outputs: ::std::vec::Vec<CellInfo>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([9, 35, 100, 72], (header, inputs, outputs))
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for ImageCell<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `rollback` function with signature `rollback(bytes32,uint64,(bytes32,uint32)[],(bytes32,uint32)[])` and selector `[226, 89, 77, 34]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "rollback",
        abi = "rollback(bytes32,uint64,(bytes32,uint32)[],(bytes32,uint32)[])"
    )]
    pub struct RollbackCall {
        pub block_hash:   [u8; 32],
        pub block_number: u64,
        pub inputs:       ::std::vec::Vec<OutPoint>,
        pub outputs:      ::std::vec::Vec<OutPoint>,
    }
    #[doc = "Container type for all input parameters for the `setState` function with signature `setState(bool)` and selector `[172, 159, 2, 34]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setState", abi = "setState(bool)")]
    pub struct SetStateCall {
        pub allow_read: bool,
    }
    #[doc = "Container type for all input parameters for the `update` function with signature `update((uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,bytes32,bytes32,bytes32,uint128,bytes32),(bytes32,uint32)[],((bytes32,uint32),(uint64,(bytes32,uint8,bytes),(bytes32,uint8,bytes)[]),bytes)[])` and selector `[9, 35, 100, 72]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(
        name = "update",
        abi = "update((uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,bytes32,bytes32,bytes32,uint128,bytes32),(bytes32,uint32)[],((bytes32,uint32),(uint64,(bytes32,uint8,bytes),(bytes32,uint8,bytes)[]),bytes)[])"
    )]
    pub struct UpdateCall {
        pub header:  Header,
        pub inputs:  ::std::vec::Vec<OutPoint>,
        pub outputs: ::std::vec::Vec<CellInfo>,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum ImageCellCalls {
        Rollback(RollbackCall),
        SetState(SetStateCall),
        Update(UpdateCall),
    }
    impl ethers::core::abi::AbiDecode for ImageCellCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <RollbackCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ImageCellCalls::Rollback(decoded));
            }
            if let Ok(decoded) =
                <SetStateCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ImageCellCalls::SetState(decoded));
            }
            if let Ok(decoded) = <UpdateCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(ImageCellCalls::Update(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for ImageCellCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                ImageCellCalls::Rollback(element) => element.encode(),
                ImageCellCalls::SetState(element) => element.encode(),
                ImageCellCalls::Update(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for ImageCellCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                ImageCellCalls::Rollback(element) => element.fmt(f),
                ImageCellCalls::SetState(element) => element.fmt(f),
                ImageCellCalls::Update(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<RollbackCall> for ImageCellCalls {
        fn from(var: RollbackCall) -> Self {
            ImageCellCalls::Rollback(var)
        }
    }
    impl ::std::convert::From<SetStateCall> for ImageCellCalls {
        fn from(var: SetStateCall) -> Self {
            ImageCellCalls::SetState(var)
        }
    }
    impl ::std::convert::From<UpdateCall> for ImageCellCalls {
        fn from(var: UpdateCall) -> Self {
            ImageCellCalls::Update(var)
        }
    }
    #[doc = "`CellInfo((bytes32,uint32),(uint64,(bytes32,uint8,bytes),(bytes32,uint8,bytes)[]),bytes)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct CellInfo {
        pub out_point: OutPoint,
        pub output:    CellOutput,
        pub data:      ethers::core::types::Bytes,
    }
    #[doc = "`CellOutput(uint64,(bytes32,uint8,bytes),(bytes32,uint8,bytes)[])`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct CellOutput {
        pub capacity: u64,
        pub lock:     Script,
        pub type_:    ::std::vec::Vec<Script>,
    }
    #[doc = "`Header(uint32,uint32,uint64,uint64,uint64,bytes32,bytes32,bytes32,bytes32,bytes32,uint128,bytes32)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
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
    #[doc = "`OutPoint(bytes32,uint32)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct OutPoint {
        pub tx_hash: [u8; 32],
        pub index:   u32,
    }
    #[doc = "`Script(bytes32,uint8,bytes)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
    )]
    pub struct Script {
        pub code_hash: [u8; 32],
        pub hash_type: u8,
        pub args:      ethers::core::types::Bytes,
    }
}
