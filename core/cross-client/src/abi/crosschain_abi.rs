pub use crosschain_mod::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod crosschain_mod {
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
    #[doc = "crosschain was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static CROSSCHAIN_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
        ethers_contract::Lazy::new(|| {
            serde_json::from_str ("[\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"metadata\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"address\",\n        \"name\": \"wCKB\",\n        \"type\": \"address\"\n      }\n    ],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"constructor\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": false,\n        \"internalType\": \"uint256\",\n        \"name\": \"minWCKB\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"ChangeMinWCKB\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": false,\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      },\n      {\n        \"components\": [\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"feeRatio\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"threshold\",\n            \"type\": \"uint256\"\n          }\n        ],\n        \"indexed\": false,\n        \"internalType\": \"struct DataType.TokenConfig\",\n        \"name\": \"config\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"ChangeTokenConfig\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"address\",\n            \"name\": \"to\",\n            \"type\": \"address\"\n          },\n          {\n            \"internalType\": \"address\",\n            \"name\": \"tokenAddress\",\n            \"type\": \"address\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"sUDTAmount\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"CKBAmount\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"txHash\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"indexed\": false,\n        \"internalType\": \"struct DataType.CKBToAxonRecord[]\",\n        \"name\": \"records\",\n        \"type\": \"tuple[]\"\n      }\n    ],\n    \"name\": \"CrossFromCKB\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": false,\n        \"internalType\": \"bytes32\",\n        \"name\": \"currentRecordHash\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"bytes32[]\",\n        \"name\": \"remainRecordsHash\",\n        \"type\": \"bytes32[]\"\n      }\n    ],\n    \"name\": \"CrossLimitRecord\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": false,\n        \"internalType\": \"string\",\n        \"name\": \"to\",\n        \"type\": \"string\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"uint256\",\n        \"name\": \"minWCKBAmount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"CrossToCKB\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": false,\n        \"internalType\": \"string\",\n        \"name\": \"to\",\n        \"type\": \"string\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"uint256\",\n        \"name\": \"minWCKBAmount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"CrossToCKBAlert\",\n    \"type\": \"event\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"AT_ADDRESS\",\n    \"outputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"\",\n        \"type\": \"address\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"typehash\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"name\": \"addMirrorToken\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"typehash\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"name\": \"addToken\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"addWhitelist\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"address\",\n            \"name\": \"to\",\n            \"type\": \"address\"\n          },\n          {\n            \"internalType\": \"address\",\n            \"name\": \"tokenAddress\",\n            \"type\": \"address\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"sUDTAmount\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"CKBAmount\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"bytes32\",\n            \"name\": \"txHash\",\n            \"type\": \"bytes32\"\n          }\n        ],\n        \"internalType\": \"struct DataType.CKBToAxonRecord[]\",\n        \"name\": \"records\",\n        \"type\": \"tuple[]\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"nonce\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"crossFromCKB\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"string\",\n        \"name\": \"to\",\n        \"type\": \"string\"\n      },\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"crossTokenToCKB\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"value\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"fee\",\n    \"outputs\": [\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"typehash\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"name\": \"getTokenAddress\",\n    \"outputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"\",\n        \"type\": \"address\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"getTokenConfig\",\n    \"outputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"feeRatio\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"threshold\",\n            \"type\": \"uint256\"\n          }\n        ],\n        \"internalType\": \"struct DataType.TokenConfig\",\n        \"name\": \"\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"getTypehash\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"getWCKBAddress\",\n    \"outputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"\",\n        \"type\": \"address\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"getWCKBMin\",\n    \"outputs\": [\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"isMirrorToken\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"isWhitelist\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"limitTxes\",\n    \"outputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"address\",\n            \"name\": \"tokenAddress\",\n            \"type\": \"address\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"amount\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"minWCKBAmount\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"string\",\n            \"name\": \"to\",\n            \"type\": \"string\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"limitSign\",\n            \"type\": \"uint256\"\n          }\n        ],\n        \"internalType\": \"struct DataType.AxonToCKBRecord[]\",\n        \"name\": \"\",\n        \"type\": \"tuple[]\"\n      },\n      {\n        \"internalType\": \"bytes32[]\",\n        \"name\": \"\",\n        \"type\": \"bytes32[]\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"string\",\n        \"name\": \"to\",\n        \"type\": \"string\"\n      }\n    ],\n    \"name\": \"lockAT\",\n    \"outputs\": [],\n    \"stateMutability\": \"payable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"mirrorTokens\",\n    \"outputs\": [\n      {\n        \"internalType\": \"address[]\",\n        \"name\": \"\",\n        \"type\": \"address[]\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"components\": [\n          {\n            \"internalType\": \"address\",\n            \"name\": \"tokenAddress\",\n            \"type\": \"address\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"amount\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"minWCKBAmount\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"string\",\n            \"name\": \"to\",\n            \"type\": \"string\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"limitSign\",\n            \"type\": \"uint256\"\n          }\n        ],\n        \"internalType\": \"struct DataType.AxonToCKBRecord\",\n        \"name\": \"record\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"removeLimitTx\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"removeWhitelist\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"token\",\n        \"type\": \"address\"\n      },\n      {\n        \"components\": [\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"feeRatio\",\n            \"type\": \"uint256\"\n          },\n          {\n            \"internalType\": \"uint256\",\n            \"name\": \"threshold\",\n            \"type\": \"uint256\"\n          }\n        ],\n        \"internalType\": \"struct DataType.TokenConfig\",\n        \"name\": \"config\",\n        \"type\": \"tuple\"\n      }\n    ],\n    \"name\": \"setTokenConfig\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"setWCKBMin\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"whitelist\",\n    \"outputs\": [\n      {\n        \"internalType\": \"address[]\",\n        \"name\": \"\",\n        \"type\": \"address[]\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  }\n]\n") . expect ("invalid abi")
        });
    pub struct crosschain<M>(ethers_contract::Contract<M>);
    impl<M> Clone for crosschain<M> {
        fn clone(&self) -> Self {
            crosschain(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for crosschain<M> {
        type Target = ethers_contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: ethers_providers::Middleware> std::fmt::Debug for crosschain<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(crosschain))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers_providers::Middleware> crosschain<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers_core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers_contract::Contract::new(address.into(), CROSSCHAIN_ABI.clone(), client).into()
        }

        #[doc = "Calls the contract's `AT_ADDRESS` (0x540f6dec) function"]
        pub fn at_address(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::Address> {
            self.0
                .method_hash([84, 15, 109, 236], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `addMirrorToken` (0x65744d24) function"]
        pub fn add_mirror_token(
            &self,
            token: ethers_core::types::Address,
            typehash: [u8; 32],
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([101, 116, 77, 36], (token, typehash))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `addToken` (0xc0c1eebc) function"]
        pub fn add_token(
            &self,
            token: ethers_core::types::Address,
            typehash: [u8; 32],
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([192, 193, 238, 188], (token, typehash))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `addWhitelist` (0xf80f5dd5) function"]
        pub fn add_whitelist(
            &self,
            token: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([248, 15, 93, 213], token)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `crossFromCKB` (0x6b23fd20) function"]
        pub fn cross_from_ckb(
            &self,
            records: ::std::vec::Vec<CkbtoAxonRecord>,
            nonce: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([107, 35, 253, 32], (records, nonce))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `crossTokenToCKB` (0xb8f564f8) function"]
        pub fn cross_token_to_ckb(
            &self,
            to: String,
            token: ethers_core::types::Address,
            amount: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([184, 245, 100, 248], (to, token, amount))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `fee` (0x9e6eda18) function"]
        pub fn fee(
            &self,
            token: ethers_core::types::Address,
            value: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([158, 110, 218, 24], (token, value))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getTokenAddress` (0xb12e4410) function"]
        pub fn get_token_address(
            &self,
            typehash: [u8; 32],
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::Address> {
            self.0
                .method_hash([177, 46, 68, 16], typehash)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getTokenConfig` (0xcb67e3b1) function"]
        pub fn get_token_config(
            &self,
            token: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, TokenConfig> {
            self.0
                .method_hash([203, 103, 227, 177], token)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getTypehash` (0xe6c8283e) function"]
        pub fn get_typehash(
            &self,
            token: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([230, 200, 40, 62], token)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getWCKBAddress` (0x2083d267) function"]
        pub fn get_wckb_address(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::Address> {
            self.0
                .method_hash([32, 131, 210, 103], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getWCKBMin` (0x19e4d989) function"]
        pub fn get_wckb_min(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([25, 228, 217, 137], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `isMirrorToken` (0xcb6ebb9b) function"]
        pub fn is_mirror_token(
            &self,
            token: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([203, 110, 187, 155], token)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `isWhitelist` (0xc683630d) function"]
        pub fn is_whitelist(
            &self,
            token: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([198, 131, 99, 13], token)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `limitTxes` (0xbf56fbd0) function"]
        pub fn limit_txes(
            &self,
        ) -> ethers_contract::builders::ContractCall<
            M,
            (::std::vec::Vec<AxonToCKBRecord>, ::std::vec::Vec<[u8; 32]>),
        > {
            self.0
                .method_hash([191, 86, 251, 208], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `lockAT` (0xdb2b749f) function"]
        pub fn lock_at(&self, to: String) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([219, 43, 116, 159], to)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `mirrorTokens` (0x9938155d) function"]
        pub fn mirror_tokens(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ::std::vec::Vec<ethers_core::types::Address>>
        {
            self.0
                .method_hash([153, 56, 21, 93], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `removeLimitTx` (0xe6193b88) function"]
        pub fn remove_limit_tx(
            &self,
            record: AxonToCKBRecord,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([230, 25, 59, 136], (record,))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `removeWhitelist` (0x78c8cda7) function"]
        pub fn remove_whitelist(
            &self,
            token: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([120, 200, 205, 167], token)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `setTokenConfig` (0x3edfdbb0) function"]
        pub fn set_token_config(
            &self,
            token: ethers_core::types::Address,
            config: TokenConfig,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([62, 223, 219, 176], (token, config))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `setWCKBMin` (0xa938d745) function"]
        pub fn set_wckb_min(
            &self,
            amount: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([169, 56, 215, 69], amount)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `whitelist` (0x93e59dc1) function"]
        pub fn whitelist(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ::std::vec::Vec<ethers_core::types::Address>>
        {
            self.0
                .method_hash([147, 229, 157, 193], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Gets the contract's `ChangeMinWCKB` event"]
        pub fn change_min_wckb_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, ChangeMinWCKBFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `ChangeTokenConfig` event"]
        pub fn change_token_config_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, ChangeTokenConfigFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `CrossFromCKB` event"]
        pub fn cross_from_ckb_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, CrossFromCKBFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `CrossLimitRecord` event"]
        pub fn cross_limit_record_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, CrossLimitRecordFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `CrossToCKB` event"]
        pub fn cross_to_ckb_filter(&self) -> ethers_contract::builders::Event<M, CrossToCKBFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `CrossToCKBAlert` event"]
        pub fn cross_to_ckb_alert_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, CrossToCKBAlertFilter> {
            self.0.event()
        }

        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers_contract::builders::Event<M, crosschainEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers_providers::Middleware> From<ethers_contract::Contract<M>> for crosschain<M> {
        fn from(contract: ethers_contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(name = "ChangeMinWCKB", abi = "ChangeMinWCKB(uint256)")]
    pub struct ChangeMinWCKBFilter {
        pub min_wckb: ethers_core::types::U256,
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(
        name = "ChangeTokenConfig",
        abi = "ChangeTokenConfig(address,(uint256,uint256))"
    )]
    pub struct ChangeTokenConfigFilter {
        pub token:  ethers_core::types::Address,
        pub config: (ethers_core::types::U256, ethers_core::types::U256),
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(
        name = "CrossFromCKB",
        abi = "CrossFromCKB((address,address,uint256,uint256,bytes32)[])"
    )]
    pub struct CrossFromCKBFilter {
        pub records: Vec<(
            ethers_core::types::Address,
            ethers_core::types::Address,
            ethers_core::types::U256,
            ethers_core::types::U256,
            [u8; 32],
        )>,
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(name = "CrossLimitRecord", abi = "CrossLimitRecord(bytes32,bytes32[])")]
    pub struct CrossLimitRecordFilter {
        pub current_record_hash: [u8; 32],
        pub remain_records_hash: Vec<[u8; 32]>,
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(
        name = "CrossToCKB",
        abi = "CrossToCKB(string,address,uint256,uint256)"
    )]
    pub struct CrossToCKBFilter {
        pub to:              String,
        pub token:           ethers_core::types::Address,
        pub amount:          ethers_core::types::U256,
        pub min_wckb_amount: ethers_core::types::U256,
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(
        name = "CrossToCKBAlert",
        abi = "CrossToCKBAlert(string,address,uint256,uint256)"
    )]
    pub struct CrossToCKBAlertFilter {
        pub to:              String,
        pub token:           ethers_core::types::Address,
        pub amount:          ethers_core::types::U256,
        pub min_wckb_amount: ethers_core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers_contract::EthAbiType)]
    pub enum crosschainEvents {
        ChangeMinWCKBFilter(ChangeMinWCKBFilter),
        ChangeTokenConfigFilter(ChangeTokenConfigFilter),
        CrossFromCKBFilter(CrossFromCKBFilter),
        CrossLimitRecordFilter(CrossLimitRecordFilter),
        CrossToCKBFilter(CrossToCKBFilter),
        CrossToCKBAlertFilter(CrossToCKBAlertFilter),
    }
    impl ethers_contract::EthLogDecode for crosschainEvents {
        fn decode_log(log: &ethers_core::abi::RawLog) -> Result<Self, ethers_core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ChangeMinWCKBFilter::decode_log(log) {
                return Ok(crosschainEvents::ChangeMinWCKBFilter(decoded));
            }
            if let Ok(decoded) = ChangeTokenConfigFilter::decode_log(log) {
                return Ok(crosschainEvents::ChangeTokenConfigFilter(decoded));
            }
            if let Ok(decoded) = CrossFromCKBFilter::decode_log(log) {
                return Ok(crosschainEvents::CrossFromCKBFilter(decoded));
            }
            if let Ok(decoded) = CrossLimitRecordFilter::decode_log(log) {
                return Ok(crosschainEvents::CrossLimitRecordFilter(decoded));
            }
            if let Ok(decoded) = CrossToCKBFilter::decode_log(log) {
                return Ok(crosschainEvents::CrossToCKBFilter(decoded));
            }
            if let Ok(decoded) = CrossToCKBAlertFilter::decode_log(log) {
                return Ok(crosschainEvents::CrossToCKBAlertFilter(decoded));
            }
            Err(ethers_core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for crosschainEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                crosschainEvents::ChangeMinWCKBFilter(element) => element.fmt(f),
                crosschainEvents::ChangeTokenConfigFilter(element) => element.fmt(f),
                crosschainEvents::CrossFromCKBFilter(element) => element.fmt(f),
                crosschainEvents::CrossLimitRecordFilter(element) => element.fmt(f),
                crosschainEvents::CrossToCKBFilter(element) => element.fmt(f),
                crosschainEvents::CrossToCKBAlertFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `AT_ADDRESS`function with signature `AT_ADDRESS()` and selector `[84, 15, 109, 236]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "AT_ADDRESS", abi = "AT_ADDRESS()")]
    pub struct AtAddressCall;
    #[doc = "Container type for all input parameters for the `addMirrorToken`function with signature `addMirrorToken(address,bytes32)` and selector `[101, 116, 77, 36]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "addMirrorToken", abi = "addMirrorToken(address,bytes32)")]
    pub struct AddMirrorTokenCall {
        pub token:    ethers_core::types::Address,
        pub typehash: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `addToken`function with signature `addToken(address,bytes32)` and selector `[192, 193, 238, 188]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "addToken", abi = "addToken(address,bytes32)")]
    pub struct AddTokenCall {
        pub token:    ethers_core::types::Address,
        pub typehash: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `addWhitelist`function with signature `addWhitelist(address)` and selector `[248, 15, 93, 213]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "addWhitelist", abi = "addWhitelist(address)")]
    pub struct AddWhitelistCall {
        pub token: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `crossFromCKB`function with signature `crossFromCKB((address,address,uint256,uint256,bytes32)[],uint256)` and selector `[107, 35, 253, 32]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(
        name = "crossFromCKB",
        abi = "crossFromCKB((address,address,uint256,uint256,bytes32)[],uint256)"
    )]
    pub struct CrossFromCKBCall {
        pub records: ::std::vec::Vec<CkbtoAxonRecord>,
        pub nonce:   ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `crossTokenToCKB`function with signature `crossTokenToCKB(string,address,uint256)` and selector `[184, 245, 100, 248]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(
        name = "crossTokenToCKB",
        abi = "crossTokenToCKB(string,address,uint256)"
    )]
    pub struct CrossTokenToCKBCall {
        pub to:     String,
        pub token:  ethers_core::types::Address,
        pub amount: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `fee`function with signature `fee(address,uint256)` and selector `[158, 110, 218, 24]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "fee", abi = "fee(address,uint256)")]
    pub struct FeeCall {
        pub token: ethers_core::types::Address,
        pub value: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `getTokenAddress`function with signature `getTokenAddress(bytes32)` and selector `[177, 46, 68, 16]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "getTokenAddress", abi = "getTokenAddress(bytes32)")]
    pub struct GetTokenAddressCall {
        pub typehash: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `getTokenConfig`function with signature `getTokenConfig(address)` and selector `[203, 103, 227, 177]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "getTokenConfig", abi = "getTokenConfig(address)")]
    pub struct GetTokenConfigCall {
        pub token: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `getTypehash`function with signature `getTypehash(address)` and selector `[230, 200, 40, 62]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "getTypehash", abi = "getTypehash(address)")]
    pub struct GetTypehashCall {
        pub token: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `getWCKBAddress`function with signature `getWCKBAddress()` and selector `[32, 131, 210, 103]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "getWCKBAddress", abi = "getWCKBAddress()")]
    pub struct GetWCKBAddressCall;
    #[doc = "Container type for all input parameters for the `getWCKBMin`function with signature `getWCKBMin()` and selector `[25, 228, 217, 137]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "getWCKBMin", abi = "getWCKBMin()")]
    pub struct GetWCKBMinCall;
    #[doc = "Container type for all input parameters for the `isMirrorToken`function with signature `isMirrorToken(address)` and selector `[203, 110, 187, 155]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "isMirrorToken", abi = "isMirrorToken(address)")]
    pub struct IsMirrorTokenCall {
        pub token: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `isWhitelist`function with signature `isWhitelist(address)` and selector `[198, 131, 99, 13]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "isWhitelist", abi = "isWhitelist(address)")]
    pub struct IsWhitelistCall {
        pub token: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `limitTxes`function with signature `limitTxes()` and selector `[191, 86, 251, 208]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "limitTxes", abi = "limitTxes()")]
    pub struct LimitTxesCall;
    #[doc = "Container type for all input parameters for the `lockAT`function with signature `lockAT(string)` and selector `[219, 43, 116, 159]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "lockAT", abi = "lockAT(string)")]
    pub struct LockATCall {
        pub to: String,
    }
    #[doc = "Container type for all input parameters for the `mirrorTokens`function with signature `mirrorTokens()` and selector `[153, 56, 21, 93]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "mirrorTokens", abi = "mirrorTokens()")]
    pub struct MirrorTokensCall;
    #[doc = "Container type for all input parameters for the `removeLimitTx`function with signature `removeLimitTx((address,uint256,uint256,string,uint256))` and selector `[230, 25, 59, 136]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(
        name = "removeLimitTx",
        abi = "removeLimitTx((address,uint256,uint256,string,uint256))"
    )]
    pub struct RemoveLimitTxCall {
        pub record: AxonToCKBRecord,
    }
    #[doc = "Container type for all input parameters for the `removeWhitelist`function with signature `removeWhitelist(address)` and selector `[120, 200, 205, 167]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "removeWhitelist", abi = "removeWhitelist(address)")]
    pub struct RemoveWhitelistCall {
        pub token: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `setTokenConfig`function with signature `setTokenConfig(address,(uint256,uint256))` and selector `[62, 223, 219, 176]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(
        name = "setTokenConfig",
        abi = "setTokenConfig(address,(uint256,uint256))"
    )]
    pub struct SetTokenConfigCall {
        pub token:  ethers_core::types::Address,
        pub config: TokenConfig,
    }
    #[doc = "Container type for all input parameters for the `setWCKBMin`function with signature `setWCKBMin(uint256)` and selector `[169, 56, 215, 69]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "setWCKBMin", abi = "setWCKBMin(uint256)")]
    pub struct SetWCKBMinCall {
        pub amount: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `whitelist`function with signature `whitelist()` and selector `[147, 229, 157, 193]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "whitelist", abi = "whitelist()")]
    pub struct WhitelistCall;
    #[derive(Debug, Clone, PartialEq, Eq, ethers_contract::EthAbiType)]
    pub enum crosschainCalls {
        AtAddress(AtAddressCall),
        AddMirrorToken(AddMirrorTokenCall),
        AddToken(AddTokenCall),
        AddWhitelist(AddWhitelistCall),
        CrossFromCKB(CrossFromCKBCall),
        CrossTokenToCKB(CrossTokenToCKBCall),
        Fee(FeeCall),
        GetTokenAddress(GetTokenAddressCall),
        GetTokenConfig(GetTokenConfigCall),
        GetTypehash(GetTypehashCall),
        GetWCKBAddress(GetWCKBAddressCall),
        GetWCKBMin(GetWCKBMinCall),
        IsMirrorToken(IsMirrorTokenCall),
        IsWhitelist(IsWhitelistCall),
        LimitTxes(LimitTxesCall),
        LockAT(LockATCall),
        MirrorTokens(MirrorTokensCall),
        RemoveLimitTx(RemoveLimitTxCall),
        RemoveWhitelist(RemoveWhitelistCall),
        SetTokenConfig(SetTokenConfigCall),
        SetWCKBMin(SetWCKBMinCall),
        Whitelist(WhitelistCall),
    }
    impl ethers_core::abi::AbiDecode for crosschainCalls {
        fn decode(data: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
            if let Ok(decoded) =
                <AtAddressCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::AtAddress(decoded));
            }
            if let Ok(decoded) =
                <AddMirrorTokenCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::AddMirrorToken(decoded));
            }
            if let Ok(decoded) =
                <AddTokenCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::AddToken(decoded));
            }
            if let Ok(decoded) =
                <AddWhitelistCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::AddWhitelist(decoded));
            }
            if let Ok(decoded) =
                <CrossFromCKBCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::CrossFromCKB(decoded));
            }
            if let Ok(decoded) =
                <CrossTokenToCKBCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::CrossTokenToCKB(decoded));
            }
            if let Ok(decoded) = <FeeCall as ethers_core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(crosschainCalls::Fee(decoded));
            }
            if let Ok(decoded) =
                <GetTokenAddressCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::GetTokenAddress(decoded));
            }
            if let Ok(decoded) =
                <GetTokenConfigCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::GetTokenConfig(decoded));
            }
            if let Ok(decoded) =
                <GetTypehashCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::GetTypehash(decoded));
            }
            if let Ok(decoded) =
                <GetWCKBAddressCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::GetWCKBAddress(decoded));
            }
            if let Ok(decoded) =
                <GetWCKBMinCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::GetWCKBMin(decoded));
            }
            if let Ok(decoded) =
                <IsMirrorTokenCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::IsMirrorToken(decoded));
            }
            if let Ok(decoded) =
                <IsWhitelistCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::IsWhitelist(decoded));
            }
            if let Ok(decoded) =
                <LimitTxesCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::LimitTxes(decoded));
            }
            if let Ok(decoded) = <LockATCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::LockAT(decoded));
            }
            if let Ok(decoded) =
                <MirrorTokensCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::MirrorTokens(decoded));
            }
            if let Ok(decoded) =
                <RemoveLimitTxCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::RemoveLimitTx(decoded));
            }
            if let Ok(decoded) =
                <RemoveWhitelistCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::RemoveWhitelist(decoded));
            }
            if let Ok(decoded) =
                <SetTokenConfigCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::SetTokenConfig(decoded));
            }
            if let Ok(decoded) =
                <SetWCKBMinCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::SetWCKBMin(decoded));
            }
            if let Ok(decoded) =
                <WhitelistCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(crosschainCalls::Whitelist(decoded));
            }
            Err(ethers_core::abi::Error::InvalidData.into())
        }
    }
    impl ethers_core::abi::AbiEncode for crosschainCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                crosschainCalls::AtAddress(element) => element.encode(),
                crosschainCalls::AddMirrorToken(element) => element.encode(),
                crosschainCalls::AddToken(element) => element.encode(),
                crosschainCalls::AddWhitelist(element) => element.encode(),
                crosschainCalls::CrossFromCKB(element) => element.encode(),
                crosschainCalls::CrossTokenToCKB(element) => element.encode(),
                crosschainCalls::Fee(element) => element.encode(),
                crosschainCalls::GetTokenAddress(element) => element.encode(),
                crosschainCalls::GetTokenConfig(element) => element.encode(),
                crosschainCalls::GetTypehash(element) => element.encode(),
                crosschainCalls::GetWCKBAddress(element) => element.encode(),
                crosschainCalls::GetWCKBMin(element) => element.encode(),
                crosschainCalls::IsMirrorToken(element) => element.encode(),
                crosschainCalls::IsWhitelist(element) => element.encode(),
                crosschainCalls::LimitTxes(element) => element.encode(),
                crosschainCalls::LockAT(element) => element.encode(),
                crosschainCalls::MirrorTokens(element) => element.encode(),
                crosschainCalls::RemoveLimitTx(element) => element.encode(),
                crosschainCalls::RemoveWhitelist(element) => element.encode(),
                crosschainCalls::SetTokenConfig(element) => element.encode(),
                crosschainCalls::SetWCKBMin(element) => element.encode(),
                crosschainCalls::Whitelist(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for crosschainCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                crosschainCalls::AtAddress(element) => element.fmt(f),
                crosschainCalls::AddMirrorToken(element) => element.fmt(f),
                crosschainCalls::AddToken(element) => element.fmt(f),
                crosschainCalls::AddWhitelist(element) => element.fmt(f),
                crosschainCalls::CrossFromCKB(element) => element.fmt(f),
                crosschainCalls::CrossTokenToCKB(element) => element.fmt(f),
                crosschainCalls::Fee(element) => element.fmt(f),
                crosschainCalls::GetTokenAddress(element) => element.fmt(f),
                crosschainCalls::GetTokenConfig(element) => element.fmt(f),
                crosschainCalls::GetTypehash(element) => element.fmt(f),
                crosschainCalls::GetWCKBAddress(element) => element.fmt(f),
                crosschainCalls::GetWCKBMin(element) => element.fmt(f),
                crosschainCalls::IsMirrorToken(element) => element.fmt(f),
                crosschainCalls::IsWhitelist(element) => element.fmt(f),
                crosschainCalls::LimitTxes(element) => element.fmt(f),
                crosschainCalls::LockAT(element) => element.fmt(f),
                crosschainCalls::MirrorTokens(element) => element.fmt(f),
                crosschainCalls::RemoveLimitTx(element) => element.fmt(f),
                crosschainCalls::RemoveWhitelist(element) => element.fmt(f),
                crosschainCalls::SetTokenConfig(element) => element.fmt(f),
                crosschainCalls::SetWCKBMin(element) => element.fmt(f),
                crosschainCalls::Whitelist(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AtAddressCall> for crosschainCalls {
        fn from(var: AtAddressCall) -> Self {
            crosschainCalls::AtAddress(var)
        }
    }
    impl ::std::convert::From<AddMirrorTokenCall> for crosschainCalls {
        fn from(var: AddMirrorTokenCall) -> Self {
            crosschainCalls::AddMirrorToken(var)
        }
    }
    impl ::std::convert::From<AddTokenCall> for crosschainCalls {
        fn from(var: AddTokenCall) -> Self {
            crosschainCalls::AddToken(var)
        }
    }
    impl ::std::convert::From<AddWhitelistCall> for crosschainCalls {
        fn from(var: AddWhitelistCall) -> Self {
            crosschainCalls::AddWhitelist(var)
        }
    }
    impl ::std::convert::From<CrossFromCKBCall> for crosschainCalls {
        fn from(var: CrossFromCKBCall) -> Self {
            crosschainCalls::CrossFromCKB(var)
        }
    }
    impl ::std::convert::From<CrossTokenToCKBCall> for crosschainCalls {
        fn from(var: CrossTokenToCKBCall) -> Self {
            crosschainCalls::CrossTokenToCKB(var)
        }
    }
    impl ::std::convert::From<FeeCall> for crosschainCalls {
        fn from(var: FeeCall) -> Self {
            crosschainCalls::Fee(var)
        }
    }
    impl ::std::convert::From<GetTokenAddressCall> for crosschainCalls {
        fn from(var: GetTokenAddressCall) -> Self {
            crosschainCalls::GetTokenAddress(var)
        }
    }
    impl ::std::convert::From<GetTokenConfigCall> for crosschainCalls {
        fn from(var: GetTokenConfigCall) -> Self {
            crosschainCalls::GetTokenConfig(var)
        }
    }
    impl ::std::convert::From<GetTypehashCall> for crosschainCalls {
        fn from(var: GetTypehashCall) -> Self {
            crosschainCalls::GetTypehash(var)
        }
    }
    impl ::std::convert::From<GetWCKBAddressCall> for crosschainCalls {
        fn from(var: GetWCKBAddressCall) -> Self {
            crosschainCalls::GetWCKBAddress(var)
        }
    }
    impl ::std::convert::From<GetWCKBMinCall> for crosschainCalls {
        fn from(var: GetWCKBMinCall) -> Self {
            crosschainCalls::GetWCKBMin(var)
        }
    }
    impl ::std::convert::From<IsMirrorTokenCall> for crosschainCalls {
        fn from(var: IsMirrorTokenCall) -> Self {
            crosschainCalls::IsMirrorToken(var)
        }
    }
    impl ::std::convert::From<IsWhitelistCall> for crosschainCalls {
        fn from(var: IsWhitelistCall) -> Self {
            crosschainCalls::IsWhitelist(var)
        }
    }
    impl ::std::convert::From<LimitTxesCall> for crosschainCalls {
        fn from(var: LimitTxesCall) -> Self {
            crosschainCalls::LimitTxes(var)
        }
    }
    impl ::std::convert::From<LockATCall> for crosschainCalls {
        fn from(var: LockATCall) -> Self {
            crosschainCalls::LockAT(var)
        }
    }
    impl ::std::convert::From<MirrorTokensCall> for crosschainCalls {
        fn from(var: MirrorTokensCall) -> Self {
            crosschainCalls::MirrorTokens(var)
        }
    }
    impl ::std::convert::From<RemoveLimitTxCall> for crosschainCalls {
        fn from(var: RemoveLimitTxCall) -> Self {
            crosschainCalls::RemoveLimitTx(var)
        }
    }
    impl ::std::convert::From<RemoveWhitelistCall> for crosschainCalls {
        fn from(var: RemoveWhitelistCall) -> Self {
            crosschainCalls::RemoveWhitelist(var)
        }
    }
    impl ::std::convert::From<SetTokenConfigCall> for crosschainCalls {
        fn from(var: SetTokenConfigCall) -> Self {
            crosschainCalls::SetTokenConfig(var)
        }
    }
    impl ::std::convert::From<SetWCKBMinCall> for crosschainCalls {
        fn from(var: SetWCKBMinCall) -> Self {
            crosschainCalls::SetWCKBMin(var)
        }
    }
    impl ::std::convert::From<WhitelistCall> for crosschainCalls {
        fn from(var: WhitelistCall) -> Self {
            crosschainCalls::Whitelist(var)
        }
    }
    #[doc = "`AxonToCKBRecord(address,uint256,uint256,string,uint256)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
    )]
    pub struct AxonToCKBRecord {
        pub token_address:   ethers_core::types::Address,
        pub amount:          ethers_core::types::U256,
        pub min_wckb_amount: ethers_core::types::U256,
        pub to:              String,
        pub limit_sign:      ethers_core::types::U256,
    }
    #[doc = "`CkbtoAxonRecord(address,address,uint256,uint256,bytes32)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
    )]
    pub struct CkbtoAxonRecord {
        pub to:            ethers_core::types::Address,
        pub token_address: ethers_core::types::Address,
        pub s_udt_amount:  ethers_core::types::U256,
        pub ckb_amount:    ethers_core::types::U256,
        pub tx_hash:       [u8; 32],
    }
    #[doc = "`TokenConfig(uint256,uint256)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract::EthAbiType,
        ethers_contract::EthAbiCodec,
    )]
    pub struct TokenConfig {
        pub fee_ratio: ethers_core::types::U256,
        pub threshold: ethers_core::types::U256,
    }
}
