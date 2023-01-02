pub use election::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod node_election {
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
    #[doc = "NodeElection was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"type\":\"function\",\"name\":\"getTotalVoteCount\",\"inputs\":[],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getVoteCount\",\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"nodeId\",\"type\":\"bytes32\"}],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"nodes\",\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"registerNode\",\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"nodeId\",\"type\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"vote\",\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"nodeId\",\"type\":\"bytes32\"}],\"outputs\":[],\"stateMutability\":\"nonpayable\"},{\"type\":\"function\",\"name\":\"votes\",\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\"}],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"event\",\"name\":\"NodeVoted\",\"inputs\":[{\"name\":\"node\",\"type\":\"bytes32\",\"indexed\":true}],\"anonymous\":false}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static NODEELECTION_ABI: ethers::contract::Lazy<ethers::core::abi::Abi> =
        ethers::contract::Lazy::new(|| {
            ethers::core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct NodeElection<M>(ethers::contract::Contract<M>);
    impl<M> Clone for NodeElection<M> {
        fn clone(&self) -> Self {
            NodeElection(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for NodeElection<M> {
        type Target = ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for NodeElection<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(NodeElection))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ethers::providers::Middleware> NodeElection<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers::contract::Contract::new(address.into(), NODEELECTION_ABI.clone(), client).into()
        }
        #[doc = "Calls the contract's `getTotalVoteCount` (0x288c72e8) function"]
        pub fn get_total_vote_count(
            &self,
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([40, 140, 114, 232], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getVoteCount` (0xa1695993) function"]
        pub fn get_vote_count(
            &self,
            node_id: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([161, 105, 89, 147], node_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `nodes` (0x1c53c280) function"]
        pub fn nodes(
            &self,
            p0: ethers::core::types::U256,
        ) -> ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([28, 83, 194, 128], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `registerNode` (0xa5bee12e) function"]
        pub fn register_node(
            &self,
            node_id: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([165, 190, 225, 46], node_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `vote` (0xa69beaba) function"]
        pub fn vote(&self, node_id: [u8; 32]) -> ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([166, 155, 234, 186], node_id)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `votes` (0x2b38cd96) function"]
        pub fn votes(
            &self,
            p0: [u8; 32],
        ) -> ethers::contract::builders::ContractCall<M, ethers::core::types::U256> {
            self.0
                .method_hash([43, 56, 205, 150], p0)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Gets the contract's `NodeVoted` event"]
        pub fn node_voted_filter(&self) -> ethers::contract::builders::Event<M, NodeVotedFilter> {
            self.0.event()
        }
        #[doc = r" Returns an [`Event`](#ethers_contract::builders::Event) builder for all events of this contract"]
        pub fn events(&self) -> ethers::contract::builders::Event<M, NodeVotedFilter> {
            self.0.event_with_filter(Default::default())
        }
    }
    impl<M: ethers::providers::Middleware> From<ethers::contract::Contract<M>> for NodeElection<M> {
        fn from(contract: ethers::contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthEvent,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethevent(name = "NodeVoted", abi = "NodeVoted(bytes32)")]
    pub struct NodeVotedFilter {
        #[ethevent(indexed)]
        pub node: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `getTotalVoteCount` function with signature `getTotalVoteCount()` and selector `[40, 140, 114, 232]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getTotalVoteCount", abi = "getTotalVoteCount()")]
    pub struct GetTotalVoteCountCall;
    #[doc = "Container type for all input parameters for the `getVoteCount` function with signature `getVoteCount(bytes32)` and selector `[161, 105, 89, 147]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "getVoteCount", abi = "getVoteCount(bytes32)")]
    pub struct GetVoteCountCall {
        pub node_id: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `nodes` function with signature `nodes(uint256)` and selector `[28, 83, 194, 128]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "nodes", abi = "nodes(uint256)")]
    pub struct NodesCall(pub ethers::core::types::U256);
    #[doc = "Container type for all input parameters for the `registerNode` function with signature `registerNode(bytes32)` and selector `[165, 190, 225, 46]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "registerNode", abi = "registerNode(bytes32)")]
    pub struct RegisterNodeCall {
        pub node_id: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `vote` function with signature `vote(bytes32)` and selector `[166, 155, 234, 186]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "vote", abi = "vote(bytes32)")]
    pub struct VoteCall {
        pub node_id: [u8; 32],
    }
    #[doc = "Container type for all input parameters for the `votes` function with signature `votes(bytes32)` and selector `[43, 56, 205, 150]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthCall,
        ethers :: contract :: EthDisplay,
        Default,
    )]
    #[ethcall(name = "votes", abi = "votes(bytes32)")]
    pub struct VotesCall(pub [u8; 32]);
    #[derive(Debug, Clone, PartialEq, Eq, ethers :: contract :: EthAbiType)]
    pub enum NodeElectionCalls {
        GetTotalVoteCount(GetTotalVoteCountCall),
        GetVoteCount(GetVoteCountCall),
        Nodes(NodesCall),
        RegisterNode(RegisterNodeCall),
        Vote(VoteCall),
        Votes(VotesCall),
    }
    impl ethers::core::abi::AbiDecode for NodeElectionCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers::core::abi::AbiError> {
            if let Ok(decoded) =
                <GetTotalVoteCountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(NodeElectionCalls::GetTotalVoteCount(decoded));
            }
            if let Ok(decoded) =
                <GetVoteCountCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(NodeElectionCalls::GetVoteCount(decoded));
            }
            if let Ok(decoded) = <NodesCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(NodeElectionCalls::Nodes(decoded));
            }
            if let Ok(decoded) =
                <RegisterNodeCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(NodeElectionCalls::RegisterNode(decoded));
            }
            if let Ok(decoded) = <VoteCall as ethers::core::abi::AbiDecode>::decode(data.as_ref()) {
                return Ok(NodeElectionCalls::Vote(decoded));
            }
            if let Ok(decoded) = <VotesCall as ethers::core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(NodeElectionCalls::Votes(decoded));
            }
            Err(ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ethers::core::abi::AbiEncode for NodeElectionCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                NodeElectionCalls::GetTotalVoteCount(element) => element.encode(),
                NodeElectionCalls::GetVoteCount(element) => element.encode(),
                NodeElectionCalls::Nodes(element) => element.encode(),
                NodeElectionCalls::RegisterNode(element) => element.encode(),
                NodeElectionCalls::Vote(element) => element.encode(),
                NodeElectionCalls::Votes(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for NodeElectionCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                NodeElectionCalls::GetTotalVoteCount(element) => element.fmt(f),
                NodeElectionCalls::GetVoteCount(element) => element.fmt(f),
                NodeElectionCalls::Nodes(element) => element.fmt(f),
                NodeElectionCalls::RegisterNode(element) => element.fmt(f),
                NodeElectionCalls::Vote(element) => element.fmt(f),
                NodeElectionCalls::Votes(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<GetTotalVoteCountCall> for NodeElectionCalls {
        fn from(var: GetTotalVoteCountCall) -> Self {
            NodeElectionCalls::GetTotalVoteCount(var)
        }
    }
    impl ::std::convert::From<GetVoteCountCall> for NodeElectionCalls {
        fn from(var: GetVoteCountCall) -> Self {
            NodeElectionCalls::GetVoteCount(var)
        }
    }
    impl ::std::convert::From<NodesCall> for NodeElectionCalls {
        fn from(var: NodesCall) -> Self {
            NodeElectionCalls::Nodes(var)
        }
    }
    impl ::std::convert::From<RegisterNodeCall> for NodeElectionCalls {
        fn from(var: RegisterNodeCall) -> Self {
            NodeElectionCalls::RegisterNode(var)
        }
    }
    impl ::std::convert::From<VoteCall> for NodeElectionCalls {
        fn from(var: VoteCall) -> Self {
            NodeElectionCalls::Vote(var)
        }
    }
    impl ::std::convert::From<VotesCall> for NodeElectionCalls {
        fn from(var: VotesCall) -> Self {
            NodeElectionCalls::Votes(var)
        }
    }
    #[doc = "Container type for all return fields from the `getTotalVoteCount` function with signature `getTotalVoteCount()` and selector `[40, 140, 114, 232]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetTotalVoteCountReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `getVoteCount` function with signature `getVoteCount(bytes32)` and selector `[161, 105, 89, 147]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct GetVoteCountReturn(pub ethers::core::types::U256);
    #[doc = "Container type for all return fields from the `nodes` function with signature `nodes(uint256)` and selector `[28, 83, 194, 128]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct NodesReturn(pub [u8; 32]);
    #[doc = "Container type for all return fields from the `votes` function with signature `votes(bytes32)` and selector `[43, 56, 205, 150]`"]
    #[derive(
        Clone,
        Debug,
        Eq,
        PartialEq,
        ethers :: contract :: EthAbiType,
        ethers :: contract :: EthAbiCodec,
        Default,
    )]
    pub struct VotesReturn(pub ethers::core::types::U256);
}
