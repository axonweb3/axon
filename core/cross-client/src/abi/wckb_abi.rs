pub use wckb_mod::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod wckb_mod {
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
    #[doc = "wckb was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static WCKB_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> = ethers_contract::Lazy::new(
        || {
            serde_json::from_str ("[\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"string\",\n        \"name\": \"name\",\n        \"type\": \"string\"\n      },\n      {\n        \"internalType\": \"string\",\n        \"name\": \"symbol\",\n        \"type\": \"string\"\n      },\n      {\n        \"internalType\": \"uint8\",\n        \"name\": \"decimals_\",\n        \"type\": \"uint8\"\n      }\n    ],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"constructor\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"owner\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"spender\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"uint256\",\n        \"name\": \"value\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"Approval\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"previousOwner\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"newOwner\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"OwnershipTransferred\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": true,\n        \"internalType\": \"bytes32\",\n        \"name\": \"role\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"bytes32\",\n        \"name\": \"previousAdminRole\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"bytes32\",\n        \"name\": \"newAdminRole\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"name\": \"RoleAdminChanged\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": true,\n        \"internalType\": \"bytes32\",\n        \"name\": \"role\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"account\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"sender\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"RoleGranted\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": true,\n        \"internalType\": \"bytes32\",\n        \"name\": \"role\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"account\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"sender\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"RoleRevoked\",\n    \"type\": \"event\"\n  },\n  {\n    \"anonymous\": false,\n    \"inputs\": [\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"from\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": true,\n        \"internalType\": \"address\",\n        \"name\": \"to\",\n        \"type\": \"address\"\n      },\n      {\n        \"indexed\": false,\n        \"internalType\": \"uint256\",\n        \"name\": \"value\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"Transfer\",\n    \"type\": \"event\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"DEFAULT_ADMIN_ROLE\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"MANAGER_ROLE\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"owner\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"address\",\n        \"name\": \"spender\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"allowance\",\n    \"outputs\": [\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"spender\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"approve\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"account\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"balanceOf\",\n    \"outputs\": [\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"from\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"burn\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"decimals\",\n    \"outputs\": [\n      {\n        \"internalType\": \"uint8\",\n        \"name\": \"\",\n        \"type\": \"uint8\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"spender\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"subtractedValue\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"decreaseAllowance\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"role\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"name\": \"getRoleAdmin\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"\",\n        \"type\": \"bytes32\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"role\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"internalType\": \"address\",\n        \"name\": \"account\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"grantRole\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"role\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"internalType\": \"address\",\n        \"name\": \"account\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"hasRole\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"spender\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"addedValue\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"increaseAllowance\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"to\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"mint\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"name\",\n    \"outputs\": [\n      {\n        \"internalType\": \"string\",\n        \"name\": \"\",\n        \"type\": \"string\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"owner\",\n    \"outputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"\",\n        \"type\": \"address\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"renounceOwnership\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"role\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"internalType\": \"address\",\n        \"name\": \"account\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"renounceRole\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes32\",\n        \"name\": \"role\",\n        \"type\": \"bytes32\"\n      },\n      {\n        \"internalType\": \"address\",\n        \"name\": \"account\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"revokeRole\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"bytes4\",\n        \"name\": \"interfaceId\",\n        \"type\": \"bytes4\"\n      }\n    ],\n    \"name\": \"supportsInterface\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"symbol\",\n    \"outputs\": [\n      {\n        \"internalType\": \"string\",\n        \"name\": \"\",\n        \"type\": \"string\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [],\n    \"name\": \"totalSupply\",\n    \"outputs\": [\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"stateMutability\": \"view\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"to\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"transfer\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"from\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"address\",\n        \"name\": \"to\",\n        \"type\": \"address\"\n      },\n      {\n        \"internalType\": \"uint256\",\n        \"name\": \"amount\",\n        \"type\": \"uint256\"\n      }\n    ],\n    \"name\": \"transferFrom\",\n    \"outputs\": [\n      {\n        \"internalType\": \"bool\",\n        \"name\": \"\",\n        \"type\": \"bool\"\n      }\n    ],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  },\n  {\n    \"inputs\": [\n      {\n        \"internalType\": \"address\",\n        \"name\": \"newOwner\",\n        \"type\": \"address\"\n      }\n    ],\n    \"name\": \"transferOwnership\",\n    \"outputs\": [],\n    \"stateMutability\": \"nonpayable\",\n    \"type\": \"function\"\n  }\n]\n") . expect ("invalid abi")
        },
    );
    pub struct wckb<M>(ethers_contract::Contract<M>);
    impl<M> Clone for wckb<M> {
        fn clone(&self) -> Self {
            wckb(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for wckb<M> {
        type Target = ethers_contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: ethers_providers::Middleware> std::fmt::Debug for wckb<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(wckb))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers_providers::Middleware> wckb<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers_core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers_contract::Contract::new(address.into(), WCKB_ABI.clone(), client).into()
        }

        #[doc = "Calls the contract's `DEFAULT_ADMIN_ROLE` (0xa217fddf) function"]
        pub fn default_admin_role(&self) -> ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([162, 23, 253, 223], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `MANAGER_ROLE` (0xec87621c) function"]
        pub fn manager_role(&self) -> ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([236, 135, 98, 28], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `allowance` (0xdd62ed3e) function"]
        pub fn allowance(
            &self,
            owner: ethers_core::types::Address,
            spender: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([221, 98, 237, 62], (owner, spender))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `approve` (0x095ea7b3) function"]
        pub fn approve(
            &self,
            spender: ethers_core::types::Address,
            amount: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([9, 94, 167, 179], (spender, amount))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `balanceOf` (0x70a08231) function"]
        pub fn balance_of(
            &self,
            account: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([112, 160, 130, 49], account)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `burn` (0x9dc29fac) function"]
        pub fn burn(
            &self,
            from: ethers_core::types::Address,
            amount: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([157, 194, 159, 172], (from, amount))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `decimals` (0x313ce567) function"]
        pub fn decimals(&self) -> ethers_contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([49, 60, 229, 103], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `decreaseAllowance` (0xa457c2d7) function"]
        pub fn decrease_allowance(
            &self,
            spender: ethers_core::types::Address,
            subtracted_value: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([164, 87, 194, 215], (spender, subtracted_value))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getRoleAdmin` (0x248a9ca3) function"]
        pub fn get_role_admin(
            &self,
            role: [u8; 32],
        ) -> ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([36, 138, 156, 163], role)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `grantRole` (0x2f2ff15d) function"]
        pub fn grant_role(
            &self,
            role: [u8; 32],
            account: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([47, 47, 241, 93], (role, account))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `hasRole` (0x91d14854) function"]
        pub fn has_role(
            &self,
            role: [u8; 32],
            account: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([145, 209, 72, 84], (role, account))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `increaseAllowance` (0x39509351) function"]
        pub fn increase_allowance(
            &self,
            spender: ethers_core::types::Address,
            added_value: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([57, 80, 147, 81], (spender, added_value))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `mint` (0x40c10f19) function"]
        pub fn mint(
            &self,
            to: ethers_core::types::Address,
            amount: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([64, 193, 15, 25], (to, amount))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `name` (0x06fdde03) function"]
        pub fn name(&self) -> ethers_contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([6, 253, 222, 3], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `owner` (0x8da5cb5b) function"]
        pub fn owner(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::Address> {
            self.0
                .method_hash([141, 165, 203, 91], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `renounceOwnership` (0x715018a6) function"]
        pub fn renounce_ownership(&self) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([113, 80, 24, 166], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `renounceRole` (0x36568abe) function"]
        pub fn renounce_role(
            &self,
            role: [u8; 32],
            account: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([54, 86, 138, 190], (role, account))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `revokeRole` (0xd547741f) function"]
        pub fn revoke_role(
            &self,
            role: [u8; 32],
            account: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([213, 71, 116, 31], (role, account))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `supportsInterface` (0x01ffc9a7) function"]
        pub fn supports_interface(
            &self,
            interface_id: [u8; 4],
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([1, 255, 201, 167], interface_id)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `symbol` (0x95d89b41) function"]
        pub fn symbol(&self) -> ethers_contract::builders::ContractCall<M, String> {
            self.0
                .method_hash([149, 216, 155, 65], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `totalSupply` (0x18160ddd) function"]
        pub fn total_supply(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([24, 22, 13, 221], ())
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `transfer` (0xa9059cbb) function"]
        pub fn transfer(
            &self,
            to: ethers_core::types::Address,
            amount: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([169, 5, 156, 187], (to, amount))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `transferFrom` (0x23b872dd) function"]
        pub fn transfer_from(
            &self,
            from: ethers_core::types::Address,
            to: ethers_core::types::Address,
            amount: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([35, 184, 114, 221], (from, to, amount))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `transferOwnership` (0xf2fde38b) function"]
        pub fn transfer_ownership(
            &self,
            new_owner: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([242, 253, 227, 139], new_owner)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Gets the contract's `Approval` event"]
        pub fn approval_filter(&self) -> ethers_contract::builders::Event<M, ApprovalFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `OwnershipTransferred` event"]
        pub fn ownership_transferred_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, OwnershipTransferredFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `RoleAdminChanged` event"]
        pub fn role_admin_changed_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, RoleAdminChangedFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `RoleGranted` event"]
        pub fn role_granted_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, RoleGrantedFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `RoleRevoked` event"]
        pub fn role_revoked_filter(
            &self,
        ) -> ethers_contract::builders::Event<M, RoleRevokedFilter> {
            self.0.event()
        }

        #[doc = "Gets the contract's `Transfer` event"]
        pub fn transfer_filter(&self) -> ethers_contract::builders::Event<M, TransferFilter> {
            self.0.event()
        }

        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers_contract::builders::Event<M, wckbEvents> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers_providers::Middleware> From<ethers_contract::Contract<M>> for wckb<M> {
        fn from(contract: ethers_contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(name = "Approval", abi = "Approval(address,address,uint256)")]
    pub struct ApprovalFilter {
        #[ethevent(indexed)]
        pub owner:   ethers_core::types::Address,
        #[ethevent(indexed)]
        pub spender: ethers_core::types::Address,
        pub value:   ethers_core::types::U256,
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        #[ethevent(indexed)]
        pub previous_owner: ethers_core::types::Address,
        #[ethevent(indexed)]
        pub new_owner:      ethers_core::types::Address,
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(
        name = "RoleAdminChanged",
        abi = "RoleAdminChanged(bytes32,bytes32,bytes32)"
    )]
    pub struct RoleAdminChangedFilter {
        #[ethevent(indexed)]
        pub role:                [u8; 32],
        #[ethevent(indexed)]
        pub previous_admin_role: [u8; 32],
        #[ethevent(indexed)]
        pub new_admin_role:      [u8; 32],
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(name = "RoleGranted", abi = "RoleGranted(bytes32,address,address)")]
    pub struct RoleGrantedFilter {
        #[ethevent(indexed)]
        pub role:    [u8; 32],
        #[ethevent(indexed)]
        pub account: ethers_core::types::Address,
        #[ethevent(indexed)]
        pub sender:  ethers_core::types::Address,
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(name = "RoleRevoked", abi = "RoleRevoked(bytes32,address,address)")]
    pub struct RoleRevokedFilter {
        #[ethevent(indexed)]
        pub role:    [u8; 32],
        #[ethevent(indexed)]
        pub account: ethers_core::types::Address,
        #[ethevent(indexed)]
        pub sender:  ethers_core::types::Address,
    }
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthEvent, ethers_contract::EthDisplay,
    )]
    #[ethevent(name = "Transfer", abi = "Transfer(address,address,uint256)")]
    pub struct TransferFilter {
        #[ethevent(indexed)]
        pub from:  ethers_core::types::Address,
        #[ethevent(indexed)]
        pub to:    ethers_core::types::Address,
        pub value: ethers_core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers_contract::EthAbiType)]
    pub enum wckbEvents {
        ApprovalFilter(ApprovalFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        RoleAdminChangedFilter(RoleAdminChangedFilter),
        RoleGrantedFilter(RoleGrantedFilter),
        RoleRevokedFilter(RoleRevokedFilter),
        TransferFilter(TransferFilter),
    }
    impl ethers_contract::EthLogDecode for wckbEvents {
        fn decode_log(log: &ethers_core::abi::RawLog) -> Result<Self, ethers_core::abi::Error>
        where
            Self: Sized,
        {
            if let Ok(decoded) = ApprovalFilter::decode_log(log) {
                return Ok(wckbEvents::ApprovalFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(wckbEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = RoleAdminChangedFilter::decode_log(log) {
                return Ok(wckbEvents::RoleAdminChangedFilter(decoded));
            }
            if let Ok(decoded) = RoleGrantedFilter::decode_log(log) {
                return Ok(wckbEvents::RoleGrantedFilter(decoded));
            }
            if let Ok(decoded) = RoleRevokedFilter::decode_log(log) {
                return Ok(wckbEvents::RoleRevokedFilter(decoded));
            }
            if let Ok(decoded) = TransferFilter::decode_log(log) {
                return Ok(wckbEvents::TransferFilter(decoded));
            }
            Err(ethers_core::abi::Error::InvalidData)
        }
    }
    impl ::std::fmt::Display for wckbEvents {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                wckbEvents::ApprovalFilter(element) => element.fmt(f),
                wckbEvents::OwnershipTransferredFilter(element) => element.fmt(f),
                wckbEvents::RoleAdminChangedFilter(element) => element.fmt(f),
                wckbEvents::RoleGrantedFilter(element) => element.fmt(f),
                wckbEvents::RoleRevokedFilter(element) => element.fmt(f),
                wckbEvents::TransferFilter(element) => element.fmt(f),
            }
        }
    }
    #[doc = "Container type for all input parameters for the `DEFAULT_ADMIN_ROLE`function with signature `DEFAULT_ADMIN_ROLE()` and selector `[162, 23, 253, 223]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "DEFAULT_ADMIN_ROLE", abi = "DEFAULT_ADMIN_ROLE()")]
    pub struct DefaultAdminRoleCall;
    #[doc = "Container type for all input parameters for the `MANAGER_ROLE`function with signature `MANAGER_ROLE()` and selector `[236, 135, 98, 28]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "MANAGER_ROLE", abi = "MANAGER_ROLE()")]
    pub struct ManagerRoleCall;
    #[doc = "Container type for all input parameters for the `allowance`function with signature `allowance(address,address)` and selector `[221, 98, 237, 62]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "allowance", abi = "allowance(address,address)")]
    pub struct AllowanceCall {
        pub owner:   ethers_core::types::Address,
        pub spender: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `approve`function with signature `approve(address,uint256)` and selector `[9, 94, 167, 179]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "approve", abi = "approve(address,uint256)")]
    pub struct ApproveCall {
        pub spender: ethers_core::types::Address,
        pub amount:  ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `balanceOf`function with signature `balanceOf(address)` and selector `[112, 160, 130, 49]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "balanceOf", abi = "balanceOf(address)")]
    pub struct BalanceOfCall {
        pub account: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `burn`function with signature `burn(address,uint256)` and selector `[157, 194, 159, 172]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "burn", abi = "burn(address,uint256)")]
    pub struct BurnCall {
        pub from:   ethers_core::types::Address,
        pub amount: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `decimals`function with signature `decimals()` and selector `[49, 60, 229, 103]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "decimals", abi = "decimals()")]
    pub struct DecimalsCall;
    #[doc = "Container type for all input parameters for the `decreaseAllowance`function with signature `decreaseAllowance(address,uint256)` and selector `[164, 87, 194, 215]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "decreaseAllowance", abi = "decreaseAllowance(address,uint256)")]
    pub struct DecreaseAllowanceCall {
        pub spender:          ethers_core::types::Address,
        pub subtracted_value: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `getRoleAdmin`function with signature `getRoleAdmin(bytes32)` and selector `[36, 138, 156, 163]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "getRoleAdmin", abi = "getRoleAdmin(bytes32)")]
    pub struct GetRoleAdminCall {
        pub role: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `grantRole`function with signature `grantRole(bytes32,address)` and selector `[47, 47, 241, 93]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "grantRole", abi = "grantRole(bytes32,address)")]
    pub struct GrantRoleCall {
        pub role:    [u8; 32],
        pub account: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `hasRole`function with signature `hasRole(bytes32,address)` and selector `[145, 209, 72, 84]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "hasRole", abi = "hasRole(bytes32,address)")]
    pub struct HasRoleCall {
        pub role:    [u8; 32],
        pub account: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `increaseAllowance`function with signature `increaseAllowance(address,uint256)` and selector `[57, 80, 147, 81]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "increaseAllowance", abi = "increaseAllowance(address,uint256)")]
    pub struct IncreaseAllowanceCall {
        pub spender:     ethers_core::types::Address,
        pub added_value: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `mint`function with signature `mint(address,uint256)` and selector `[64, 193, 15, 25]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "mint", abi = "mint(address,uint256)")]
    pub struct MintCall {
        pub to:     ethers_core::types::Address,
        pub amount: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `name`function with signature `name()` and selector `[6, 253, 222, 3]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "name", abi = "name()")]
    pub struct NameCall;
    #[doc = "Container type for all input parameters for the `owner`function with signature `owner()` and selector `[141, 165, 203, 91]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "owner", abi = "owner()")]
    pub struct OwnerCall;
    #[doc = "Container type for all input parameters for the `renounceOwnership`function with signature `renounceOwnership()` and selector `[113, 80, 24, 166]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "renounceOwnership", abi = "renounceOwnership()")]
    pub struct RenounceOwnershipCall;
    #[doc = "Container type for all input parameters for the `renounceRole`function with signature `renounceRole(bytes32,address)` and selector `[54, 86, 138, 190]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "renounceRole", abi = "renounceRole(bytes32,address)")]
    pub struct RenounceRoleCall {
        pub role:    [u8; 32],
        pub account: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `revokeRole`function with signature `revokeRole(bytes32,address)` and selector `[213, 71, 116, 31]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "revokeRole", abi = "revokeRole(bytes32,address)")]
    pub struct RevokeRoleCall {
        pub role:    [u8; 32],
        pub account: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `supportsInterface`function with signature `supportsInterface(bytes4)` and selector `[1, 255, 201, 167]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "supportsInterface", abi = "supportsInterface(bytes4)")]
    pub struct SupportsInterfaceCall {
        pub interface_id: [u8; 4],
    }
    #[doc = "Container type for all input parameters for the `symbol`function with signature `symbol()` and selector `[149, 216, 155, 65]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "symbol", abi = "symbol()")]
    pub struct SymbolCall;
    #[doc = "Container type for all input parameters for the `totalSupply`function with signature `totalSupply()` and selector `[24, 22, 13, 221]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "totalSupply", abi = "totalSupply()")]
    pub struct TotalSupplyCall;
    #[doc = "Container type for all input parameters for the `transfer`function with signature `transfer(address,uint256)` and selector `[169, 5, 156, 187]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "transfer", abi = "transfer(address,uint256)")]
    pub struct TransferCall {
        pub to:     ethers_core::types::Address,
        pub amount: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `transferFrom`function with signature `transferFrom(address,address,uint256)` and selector `[35, 184, 114, 221]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "transferFrom", abi = "transferFrom(address,address,uint256)")]
    pub struct TransferFromCall {
        pub from:   ethers_core::types::Address,
        pub to:     ethers_core::types::Address,
        pub amount: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `transferOwnership`function with signature `transferOwnership(address)` and selector `[242, 253, 227, 139]`"]
    #[derive(
        Clone, Debug, Default, Eq, PartialEq, ethers_contract::EthCall, ethers_contract::EthDisplay,
    )]
    #[ethcall(name = "transferOwnership", abi = "transferOwnership(address)")]
    pub struct TransferOwnershipCall {
        pub new_owner: ethers_core::types::Address,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers_contract::EthAbiType)]
    pub enum wckbCalls {
        DefaultAdminRole(DefaultAdminRoleCall),
        ManagerRole(ManagerRoleCall),
        Allowance(AllowanceCall),
        Approve(ApproveCall),
        BalanceOf(BalanceOfCall),
        Burn(BurnCall),
        Decimals(DecimalsCall),
        DecreaseAllowance(DecreaseAllowanceCall),
        GetRoleAdmin(GetRoleAdminCall),
        GrantRole(GrantRoleCall),
        HasRole(HasRoleCall),
        IncreaseAllowance(IncreaseAllowanceCall),
        Mint(MintCall),
        Name(NameCall),
        Owner(OwnerCall),
        RenounceOwnership(RenounceOwnershipCall),
        RenounceRole(RenounceRoleCall),
        RevokeRole(RevokeRoleCall),
        SupportsInterface(SupportsInterfaceCall),
        Symbol(SymbolCall),
        TotalSupply(TotalSupplyCall),
        Transfer(TransferCall),
        TransferFrom(TransferFromCall),
        TransferOwnership(TransferOwnershipCall),
    }
    impl ethers_core::abi::AbiDecode for wckbCalls {
        fn decode(data: impl AsRef<[u8]>) -> Result<Self, ethers_core::abi::AbiError> {
            if let Ok(decoded) =
                <DefaultAdminRoleCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::DefaultAdminRole(decoded));
            }
            if let Ok(decoded) =
                <ManagerRoleCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::ManagerRole(decoded));
            }
            if let Ok(decoded) =
                <AllowanceCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::Allowance(decoded));
            }
            if let Ok(decoded) = <ApproveCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::Approve(decoded));
            }
            if let Ok(decoded) =
                <BalanceOfCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::BalanceOf(decoded));
            }
            if let Ok(decoded) = <BurnCall as ethers_core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(wckbCalls::Burn(decoded));
            }
            if let Ok(decoded) =
                <DecimalsCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::Decimals(decoded));
            }
            if let Ok(decoded) =
                <DecreaseAllowanceCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::DecreaseAllowance(decoded));
            }
            if let Ok(decoded) =
                <GetRoleAdminCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::GetRoleAdmin(decoded));
            }
            if let Ok(decoded) =
                <GrantRoleCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::GrantRole(decoded));
            }
            if let Ok(decoded) = <HasRoleCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::HasRole(decoded));
            }
            if let Ok(decoded) =
                <IncreaseAllowanceCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::IncreaseAllowance(decoded));
            }
            if let Ok(decoded) = <MintCall as ethers_core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(wckbCalls::Mint(decoded));
            }
            if let Ok(decoded) = <NameCall as ethers_core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(wckbCalls::Name(decoded));
            }
            if let Ok(decoded) = <OwnerCall as ethers_core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(wckbCalls::Owner(decoded));
            }
            if let Ok(decoded) =
                <RenounceOwnershipCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::RenounceOwnership(decoded));
            }
            if let Ok(decoded) =
                <RenounceRoleCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::RenounceRole(decoded));
            }
            if let Ok(decoded) =
                <RevokeRoleCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::RevokeRole(decoded));
            }
            if let Ok(decoded) =
                <SupportsInterfaceCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::SupportsInterface(decoded));
            }
            if let Ok(decoded) = <SymbolCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::Symbol(decoded));
            }
            if let Ok(decoded) =
                <TotalSupplyCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::TotalSupply(decoded));
            }
            if let Ok(decoded) =
                <TransferCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::Transfer(decoded));
            }
            if let Ok(decoded) =
                <TransferFromCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::TransferFrom(decoded));
            }
            if let Ok(decoded) =
                <TransferOwnershipCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(wckbCalls::TransferOwnership(decoded));
            }
            Err(ethers_core::abi::Error::InvalidData.into())
        }
    }
    impl ethers_core::abi::AbiEncode for wckbCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                wckbCalls::DefaultAdminRole(element) => element.encode(),
                wckbCalls::ManagerRole(element) => element.encode(),
                wckbCalls::Allowance(element) => element.encode(),
                wckbCalls::Approve(element) => element.encode(),
                wckbCalls::BalanceOf(element) => element.encode(),
                wckbCalls::Burn(element) => element.encode(),
                wckbCalls::Decimals(element) => element.encode(),
                wckbCalls::DecreaseAllowance(element) => element.encode(),
                wckbCalls::GetRoleAdmin(element) => element.encode(),
                wckbCalls::GrantRole(element) => element.encode(),
                wckbCalls::HasRole(element) => element.encode(),
                wckbCalls::IncreaseAllowance(element) => element.encode(),
                wckbCalls::Mint(element) => element.encode(),
                wckbCalls::Name(element) => element.encode(),
                wckbCalls::Owner(element) => element.encode(),
                wckbCalls::RenounceOwnership(element) => element.encode(),
                wckbCalls::RenounceRole(element) => element.encode(),
                wckbCalls::RevokeRole(element) => element.encode(),
                wckbCalls::SupportsInterface(element) => element.encode(),
                wckbCalls::Symbol(element) => element.encode(),
                wckbCalls::TotalSupply(element) => element.encode(),
                wckbCalls::Transfer(element) => element.encode(),
                wckbCalls::TransferFrom(element) => element.encode(),
                wckbCalls::TransferOwnership(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for wckbCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                wckbCalls::DefaultAdminRole(element) => element.fmt(f),
                wckbCalls::ManagerRole(element) => element.fmt(f),
                wckbCalls::Allowance(element) => element.fmt(f),
                wckbCalls::Approve(element) => element.fmt(f),
                wckbCalls::BalanceOf(element) => element.fmt(f),
                wckbCalls::Burn(element) => element.fmt(f),
                wckbCalls::Decimals(element) => element.fmt(f),
                wckbCalls::DecreaseAllowance(element) => element.fmt(f),
                wckbCalls::GetRoleAdmin(element) => element.fmt(f),
                wckbCalls::GrantRole(element) => element.fmt(f),
                wckbCalls::HasRole(element) => element.fmt(f),
                wckbCalls::IncreaseAllowance(element) => element.fmt(f),
                wckbCalls::Mint(element) => element.fmt(f),
                wckbCalls::Name(element) => element.fmt(f),
                wckbCalls::Owner(element) => element.fmt(f),
                wckbCalls::RenounceOwnership(element) => element.fmt(f),
                wckbCalls::RenounceRole(element) => element.fmt(f),
                wckbCalls::RevokeRole(element) => element.fmt(f),
                wckbCalls::SupportsInterface(element) => element.fmt(f),
                wckbCalls::Symbol(element) => element.fmt(f),
                wckbCalls::TotalSupply(element) => element.fmt(f),
                wckbCalls::Transfer(element) => element.fmt(f),
                wckbCalls::TransferFrom(element) => element.fmt(f),
                wckbCalls::TransferOwnership(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<DefaultAdminRoleCall> for wckbCalls {
        fn from(var: DefaultAdminRoleCall) -> Self {
            wckbCalls::DefaultAdminRole(var)
        }
    }
    impl ::std::convert::From<ManagerRoleCall> for wckbCalls {
        fn from(var: ManagerRoleCall) -> Self {
            wckbCalls::ManagerRole(var)
        }
    }
    impl ::std::convert::From<AllowanceCall> for wckbCalls {
        fn from(var: AllowanceCall) -> Self {
            wckbCalls::Allowance(var)
        }
    }
    impl ::std::convert::From<ApproveCall> for wckbCalls {
        fn from(var: ApproveCall) -> Self {
            wckbCalls::Approve(var)
        }
    }
    impl ::std::convert::From<BalanceOfCall> for wckbCalls {
        fn from(var: BalanceOfCall) -> Self {
            wckbCalls::BalanceOf(var)
        }
    }
    impl ::std::convert::From<BurnCall> for wckbCalls {
        fn from(var: BurnCall) -> Self {
            wckbCalls::Burn(var)
        }
    }
    impl ::std::convert::From<DecimalsCall> for wckbCalls {
        fn from(var: DecimalsCall) -> Self {
            wckbCalls::Decimals(var)
        }
    }
    impl ::std::convert::From<DecreaseAllowanceCall> for wckbCalls {
        fn from(var: DecreaseAllowanceCall) -> Self {
            wckbCalls::DecreaseAllowance(var)
        }
    }
    impl ::std::convert::From<GetRoleAdminCall> for wckbCalls {
        fn from(var: GetRoleAdminCall) -> Self {
            wckbCalls::GetRoleAdmin(var)
        }
    }
    impl ::std::convert::From<GrantRoleCall> for wckbCalls {
        fn from(var: GrantRoleCall) -> Self {
            wckbCalls::GrantRole(var)
        }
    }
    impl ::std::convert::From<HasRoleCall> for wckbCalls {
        fn from(var: HasRoleCall) -> Self {
            wckbCalls::HasRole(var)
        }
    }
    impl ::std::convert::From<IncreaseAllowanceCall> for wckbCalls {
        fn from(var: IncreaseAllowanceCall) -> Self {
            wckbCalls::IncreaseAllowance(var)
        }
    }
    impl ::std::convert::From<MintCall> for wckbCalls {
        fn from(var: MintCall) -> Self {
            wckbCalls::Mint(var)
        }
    }
    impl ::std::convert::From<NameCall> for wckbCalls {
        fn from(var: NameCall) -> Self {
            wckbCalls::Name(var)
        }
    }
    impl ::std::convert::From<OwnerCall> for wckbCalls {
        fn from(var: OwnerCall) -> Self {
            wckbCalls::Owner(var)
        }
    }
    impl ::std::convert::From<RenounceOwnershipCall> for wckbCalls {
        fn from(var: RenounceOwnershipCall) -> Self {
            wckbCalls::RenounceOwnership(var)
        }
    }
    impl ::std::convert::From<RenounceRoleCall> for wckbCalls {
        fn from(var: RenounceRoleCall) -> Self {
            wckbCalls::RenounceRole(var)
        }
    }
    impl ::std::convert::From<RevokeRoleCall> for wckbCalls {
        fn from(var: RevokeRoleCall) -> Self {
            wckbCalls::RevokeRole(var)
        }
    }
    impl ::std::convert::From<SupportsInterfaceCall> for wckbCalls {
        fn from(var: SupportsInterfaceCall) -> Self {
            wckbCalls::SupportsInterface(var)
        }
    }
    impl ::std::convert::From<SymbolCall> for wckbCalls {
        fn from(var: SymbolCall) -> Self {
            wckbCalls::Symbol(var)
        }
    }
    impl ::std::convert::From<TotalSupplyCall> for wckbCalls {
        fn from(var: TotalSupplyCall) -> Self {
            wckbCalls::TotalSupply(var)
        }
    }
    impl ::std::convert::From<TransferCall> for wckbCalls {
        fn from(var: TransferCall) -> Self {
            wckbCalls::Transfer(var)
        }
    }
    impl ::std::convert::From<TransferFromCall> for wckbCalls {
        fn from(var: TransferFromCall) -> Self {
            wckbCalls::TransferFrom(var)
        }
    }
    impl ::std::convert::From<TransferOwnershipCall> for wckbCalls {
        fn from(var: TransferOwnershipCall) -> Self {
            wckbCalls::TransferOwnership(var)
        }
    }
}
