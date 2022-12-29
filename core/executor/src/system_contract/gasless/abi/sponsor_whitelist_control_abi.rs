pub use sponsor_whitelist_control::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod sponsor_whitelist_control {
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
    #[doc = "SponsorWhitelistControl was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"type\":\"function\",\"name\":\"addPrivilege\",\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"users\",\"type\":\"address[]\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"getSponsorForGas\",\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddr\",\"type\":\"address\"}],\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getSponsoredBalanceForGas\",\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddr\",\"type\":\"address\"}],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getSponsoredGasFeeUpperBound\",\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddr\",\"type\":\"address\"}],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isAllWhitelisted\",\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddr\",\"type\":\"address\"}],\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"isWhitelisted\",\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddr\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"user\",\"type\":\"address\"}],\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"removePrivilege\",\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"users\",\"type\":\"address[]\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"setSponsorForGas\",\"inputs\":[{\"internalType\":\"address\",\"name\":\"contractAddr\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"upperBound\",\"type\":\"uint256\"}],\"outputs\":[],\"stateMutability\":\"payable\"}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static SPONSORWHITELISTCONTROL_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct SponsorWhitelistControl<M>(ethers::contract::Contract<M>);
    impl<M> Clone for SponsorWhitelistControl<M> {
        fn clone(&self) -> Self {
            SponsorWhitelistControl(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for SponsorWhitelistControl<M> {
        type Target = ethers::contract::Contract<M>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for SponsorWhitelistControl<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(SponsorWhitelistControl))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> SponsorWhitelistControl<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(
                address.into(),
                SPONSORWHITELISTCONTROL_ABI.clone(),
                client,
            )
            .into()
        }

        #[doc = "Calls the contract's `addPrivilege` (0x10128d3e) function"]
        pub fn add_privilege(
            &self,
            users: ::std::vec::Vec<ethers::core::types::Address>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([16, 18, 141, 62], users)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getSponsorForGas` (0x33a1af31) function"]
        pub fn get_sponsor_for_gas(
            &self,
            contract_addr: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::Address> {
            self.0
                .method_hash([51, 161, 175, 49], contract_addr)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getSponsoredBalanceForGas` (0xb3b28fac) function"]
        pub fn get_sponsored_balance_for_gas(
            &self,
            contract_addr: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([179, 178, 143, 172], contract_addr)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `getSponsoredGasFeeUpperBound` (0xd665f9dd) function"]
        pub fn get_sponsored_gas_fee_upper_bound(
            &self,
            contract_addr: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([214, 101, 249, 221], contract_addr)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `isAllWhitelisted` (0x79b47faa) function"]
        pub fn is_all_whitelisted(
            &self,
            contract_addr: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([121, 180, 127, 170], contract_addr)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `isWhitelisted` (0xb6b35272) function"]
        pub fn is_whitelisted(
            &self,
            contract_addr: ethers::core::types::Address,
            user: ethers::core::types::Address,
        ) -> ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([182, 179, 82, 114], (contract_addr, user))
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `removePrivilege` (0xd2932db6) function"]
        pub fn remove_privilege(
            &self,
            users: ::std::vec::Vec<ethers::core::types::Address>,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([210, 147, 45, 182], users)
                .expect("method not found (this should never happen)")
        }

        #[doc = "Calls the contract's `setSponsorForGas` (0x3e3e6428) function"]
        pub fn set_sponsor_for_gas(
            &self,
            contract_addr: ethers::core::types::Address,
            upper_bound: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([62, 62, 100, 40], (contract_addr, upper_bound))
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>>
        for SponsorWhitelistControl<M>
    {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `addPrivilege` function with signature `addPrivilege(address[])` and selector `0x10128d3e`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "addPrivilege", abi = "addPrivilege(address[])")]
    pub struct AddPrivilegeCall {
        pub users: ::std::vec::Vec<ethers::core::types::Address>,
    }
    #[doc = "Container type for all input parameters for the `getSponsorForGas` function with signature `getSponsorForGas(address)` and selector `0x33a1af31`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getSponsorForGas", abi = "getSponsorForGas(address)")]
    pub struct GetSponsorForGasCall {
        pub contract_addr: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `getSponsoredBalanceForGas` function with signature `getSponsoredBalanceForGas(address)` and selector `0xb3b28fac`"]
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
        name = "getSponsoredBalanceForGas",
        abi = "getSponsoredBalanceForGas(address)"
    )]
    pub struct GetSponsoredBalanceForGasCall {
        pub contract_addr: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `getSponsoredGasFeeUpperBound` function with signature `getSponsoredGasFeeUpperBound(address)` and selector `0xd665f9dd`"]
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
        name = "getSponsoredGasFeeUpperBound",
        abi = "getSponsoredGasFeeUpperBound(address)"
    )]
    pub struct GetSponsoredGasFeeUpperBoundCall {
        pub contract_addr: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `isAllWhitelisted` function with signature `isAllWhitelisted(address)` and selector `0x79b47faa`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isAllWhitelisted", abi = "isAllWhitelisted(address)")]
    pub struct IsAllWhitelistedCall {
        pub contract_addr: ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `isWhitelisted` function with signature `isWhitelisted(address,address)` and selector `0xb6b35272`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "isWhitelisted", abi = "isWhitelisted(address,address)")]
    pub struct IsWhitelistedCall {
        pub contract_addr: ethers::core::types::Address,
        pub user:          ethers::core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `removePrivilege` function with signature `removePrivilege(address[])` and selector `0xd2932db6`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "removePrivilege", abi = "removePrivilege(address[])")]
    pub struct RemovePrivilegeCall {
        pub users: ::std::vec::Vec<ethers::core::types::Address>,
    }
    #[doc = "Container type for all input parameters for the `setSponsorForGas` function with signature `setSponsorForGas(address,uint256)` and selector `0x3e3e6428`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "setSponsorForGas", abi = "setSponsorForGas(address,uint256)")]
    pub struct SetSponsorForGasCall {
        pub contract_addr: ethers::core::types::Address,
        pub upper_bound:   ethers::core::types::U256,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum SponsorWhitelistControlCalls {
        AddPrivilege(AddPrivilegeCall),
        GetSponsorForGas(GetSponsorForGasCall),
        GetSponsoredBalanceForGas(GetSponsoredBalanceForGasCall),
        GetSponsoredGasFeeUpperBound(GetSponsoredGasFeeUpperBoundCall),
        IsAllWhitelisted(IsAllWhitelistedCall),
        IsWhitelisted(IsWhitelistedCall),
        RemovePrivilege(RemovePrivilegeCall),
        SetSponsorForGas(SetSponsorForGasCall),
    }
    impl ethers::core::abi::AbiDecode for SponsorWhitelistControlCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <AddPrivilegeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(SponsorWhitelistControlCalls::AddPrivilege(decoded));
            }
            if let Ok(decoded) =
                <GetSponsorForGasCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(SponsorWhitelistControlCalls::GetSponsorForGas(decoded));
            }
            if let Ok(decoded) =
                <GetSponsoredBalanceForGasCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(SponsorWhitelistControlCalls::GetSponsoredBalanceForGas(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <GetSponsoredGasFeeUpperBoundCall as ethers::core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(SponsorWhitelistControlCalls::GetSponsoredGasFeeUpperBound(
                    decoded,
                ));
            }
            if let Ok(decoded) =
                <IsAllWhitelistedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(SponsorWhitelistControlCalls::IsAllWhitelisted(decoded));
            }
            if let Ok(decoded) =
                <IsWhitelistedCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(SponsorWhitelistControlCalls::IsWhitelisted(decoded));
            }
            if let Ok(decoded) =
                <RemovePrivilegeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(SponsorWhitelistControlCalls::RemovePrivilege(decoded));
            }
            if let Ok(decoded) =
                <SetSponsorForGasCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(SponsorWhitelistControlCalls::SetSponsorForGas(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for SponsorWhitelistControlCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                SponsorWhitelistControlCalls::AddPrivilege(element) => element.encode(),
                SponsorWhitelistControlCalls::GetSponsorForGas(element) => element.encode(),
                SponsorWhitelistControlCalls::GetSponsoredBalanceForGas(element) => {
                    element.encode()
                }
                SponsorWhitelistControlCalls::GetSponsoredGasFeeUpperBound(element) => {
                    element.encode()
                }
                SponsorWhitelistControlCalls::IsAllWhitelisted(element) => element.encode(),
                SponsorWhitelistControlCalls::IsWhitelisted(element) => element.encode(),
                SponsorWhitelistControlCalls::RemovePrivilege(element) => element.encode(),
                SponsorWhitelistControlCalls::SetSponsorForGas(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for SponsorWhitelistControlCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SponsorWhitelistControlCalls::AddPrivilege(element) => element.fmt(f),
                SponsorWhitelistControlCalls::GetSponsorForGas(element) => element.fmt(f),
                SponsorWhitelistControlCalls::GetSponsoredBalanceForGas(element) => element.fmt(f),
                SponsorWhitelistControlCalls::GetSponsoredGasFeeUpperBound(element) => {
                    element.fmt(f)
                }
                SponsorWhitelistControlCalls::IsAllWhitelisted(element) => element.fmt(f),
                SponsorWhitelistControlCalls::IsWhitelisted(element) => element.fmt(f),
                SponsorWhitelistControlCalls::RemovePrivilege(element) => element.fmt(f),
                SponsorWhitelistControlCalls::SetSponsorForGas(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AddPrivilegeCall> for SponsorWhitelistControlCalls {
        fn from(var: AddPrivilegeCall) -> Self {
            SponsorWhitelistControlCalls::AddPrivilege(var)
        }
    }
    impl ::std::convert::From<GetSponsorForGasCall> for SponsorWhitelistControlCalls {
        fn from(var: GetSponsorForGasCall) -> Self {
            SponsorWhitelistControlCalls::GetSponsorForGas(var)
        }
    }
    impl ::std::convert::From<GetSponsoredBalanceForGasCall> for SponsorWhitelistControlCalls {
        fn from(var: GetSponsoredBalanceForGasCall) -> Self {
            SponsorWhitelistControlCalls::GetSponsoredBalanceForGas(var)
        }
    }
    impl ::std::convert::From<GetSponsoredGasFeeUpperBoundCall> for SponsorWhitelistControlCalls {
        fn from(var: GetSponsoredGasFeeUpperBoundCall) -> Self {
            SponsorWhitelistControlCalls::GetSponsoredGasFeeUpperBound(var)
        }
    }
    impl ::std::convert::From<IsAllWhitelistedCall> for SponsorWhitelistControlCalls {
        fn from(var: IsAllWhitelistedCall) -> Self {
            SponsorWhitelistControlCalls::IsAllWhitelisted(var)
        }
    }
    impl ::std::convert::From<IsWhitelistedCall> for SponsorWhitelistControlCalls {
        fn from(var: IsWhitelistedCall) -> Self {
            SponsorWhitelistControlCalls::IsWhitelisted(var)
        }
    }
    impl ::std::convert::From<RemovePrivilegeCall> for SponsorWhitelistControlCalls {
        fn from(var: RemovePrivilegeCall) -> Self {
            SponsorWhitelistControlCalls::RemovePrivilege(var)
        }
    }
    impl ::std::convert::From<SetSponsorForGasCall> for SponsorWhitelistControlCalls {
        fn from(var: SetSponsorForGasCall) -> Self {
            SponsorWhitelistControlCalls::SetSponsorForGas(var)
        }
    }
    #[doc = "Container type for all return fields from the `getSponsorForGas` function with signature `getSponsorForGas(address)` and selector `0x33a1af31`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetSponsorForGasReturn(pub ethers::core::types::Address);
    #[doc = "Container type for all return fields from the `getSponsoredBalanceForGas` function with signature `getSponsoredBalanceForGas(address)` and selector `0xb3b28fac`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetSponsoredBalanceForGasReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `getSponsoredGasFeeUpperBound` function with signature `getSponsoredGasFeeUpperBound(address)` and selector `0xd665f9dd`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetSponsoredGasFeeUpperBoundReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `isAllWhitelisted` function with signature `isAllWhitelisted(address)` and selector `0x79b47faa`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsAllWhitelistedReturn(pub bool);
    #[doc = "Container type for all return fields from the `isWhitelisted` function with signature `isWhitelisted(address,address)` and selector `0xb6b35272`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct IsWhitelistedReturn(pub bool);
}
