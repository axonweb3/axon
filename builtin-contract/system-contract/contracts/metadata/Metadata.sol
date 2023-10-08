// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

library MetadataType {
    struct MetadataVersion {
        uint64 start;
        uint64 end;
    }

    struct ValidatorExtend {
        bytes bls_pub_key;
        bytes pub_key;
        address address_;
        uint32 propose_weight;
        uint32 vote_weight;
    }

    struct Metadata {
        MetadataVersion version;
        uint64 epoch;
        ValidatorExtend[] verifier_list;
        ProposeCount[] propose_counter;
        ConsensusConfig consensus_config;
    }

    struct ProposeCount {
        address address_;
        uint64 count;
    }

    struct ConsensusConfig {
        uint64 propose_ratio;
        uint64 prevote_ratio;
        uint64 precommit_ratio;
        uint64 brake_ratio;
        uint64 tx_num_limit;
        uint64 max_tx_size;
        uint64 gas_limit;
        uint64 gas_price;
        uint64 interval;
    }

    struct CkbRelatedInfo {
        bytes32 metadata_type_id;
        bytes32 checkpoint_type_id;
        bytes32 xudt_args;
        bytes32 stake_smt_type_id;
        bytes32 delegate_smt_type_id;
        bytes32 reward_smt_type_id;
    }
}

// **Notice**
// This solidity contract only defines the interface of metadata contract. The real
// implementation is in `core/executor/src/system_contract/metadata`.
interface MetadataManager {
    function appendMetadata(MetadataType.Metadata memory metadata) external;

    function updateConsensusConfig(MetadataType.ConsensusConfig memory config) external;

    function setCkbRelatedInfo(MetadataType.CkbRelatedInfo memory info) external;
}
