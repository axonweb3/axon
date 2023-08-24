// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.7.0;

// import "hardhat/console.sol";

contract MetadataManager {
    uint64 constant U64_MAX = 2 ** 64 - 1;

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

    // to store all metadata with epoch as key
    mapping(uint64 => Metadata) metadata_set;

    // to identify current highest epoch number
    uint64 highest_epoch;

    function construct() public {
        highest_epoch = U64_MAX;
    }

    // push new metadata into `metadata_set`
    function appendMetadata(Metadata memory metadata) public {
        require(metadata.epoch >= 0, "fatal/invalid epoch");

        bool find_sender = false;
        for (uint256 i = 0; i < metadata.verifier_list.length; i++) {
            if (metadata.verifier_list[i].address_ == msg.sender) {
                find_sender = true;
                break;
            }
        }
        require(find_sender, "fatal/verifier_list has no sender");

        uint64 epoch = metadata.epoch;
        if (highest_epoch != U64_MAX) {
            require(highest_epoch + 1 == epoch, "fatal/discontinuous epoch");
            require(
                metadata.version.start ==
                    metadata_set[highest_epoch].version.end + 1,
                "fatal/discontinuous version"
            );
        }

        Metadata storage target = metadata_set[epoch];
        target.version = metadata.version;
        target.epoch = metadata.epoch;
        target.consensus_config.gas_limit = metadata.consensus_config.gas_limit;
        target.consensus_config.gas_price = metadata.consensus_config.gas_price;
        target.consensus_config.interval = metadata.consensus_config.interval;
        target.consensus_config.propose_ratio = metadata
            .consensus_config
            .propose_ratio;
        target.consensus_config.prevote_ratio = metadata
            .consensus_config
            .prevote_ratio;
        target.consensus_config.precommit_ratio = metadata
            .consensus_config
            .precommit_ratio;
        target.consensus_config.brake_ratio = metadata
            .consensus_config
            .brake_ratio;
        target.consensus_config.tx_num_limit = metadata
            .consensus_config
            .tx_num_limit;
        target.consensus_config.max_tx_size = metadata
            .consensus_config
            .max_tx_size;
        for (uint256 i = 0; i < metadata.propose_counter.length; i++) {
            target.propose_counter.push(metadata.propose_counter[i]);
        }
        for (uint256 i = 0; i < metadata.verifier_list.length; i++) {
            target.verifier_list.push(metadata.verifier_list[i]);
        }
        highest_epoch = epoch;
    }

    // update current consensus_config
    function updateConsensusConfig(ConsensusConfig memory config) public view {
        Metadata memory highest_metadata = metadata_set[highest_epoch];

        bool find_sender = false;
        for (uint256 i = 0; i < highest_metadata.verifier_list.length; i++) {
            if (highest_metadata.verifier_list[i].address_ == msg.sender) {
                find_sender = true;
                break;
            }
        }
        require(find_sender, "fatal/verifier_list has no sender");
        highest_metadata.consensus_config = config;
    }

    // get metadata from `metadata_set` by epoch
    function getMetadata(uint64 epoch) public view returns (Metadata memory) {
        Metadata memory metadata = metadata_set[epoch];
        require(
            metadata.consensus_config.gas_limit != 0,
            "fatal/non-indexed epoch"
        );
        return metadata;
    }

    function setCkbRelatedInfo(
        CkbRelatedInfo memory ckbRelatedInfo
    ) public view {}
}
