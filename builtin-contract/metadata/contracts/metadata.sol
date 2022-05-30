// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.7.0;

// import "hardhat/console.sol";

contract MetadataManager {
    uint64 U64_MAX = 2**64 - 1;

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

    struct CrossToken {
        address[] tokens;
        bytes32[] typeHashes;
    }

    struct Metadata {
        MetadataVersion version;
        uint64 epoch;
        uint64 gas_limit;
        uint64 gas_price;
        uint64 interval;
        ValidatorExtend[] verifier_list;
        uint64 propose_ratio;
        uint64 prevote_ratio;
        uint64 precommit_ratio;
        uint64 brake_ratio;
        uint64 tx_num_limit;
        uint64 max_tx_size;
        bytes32 last_checkpoint_block_hash;
        CrossToken crossToken;
    }

    // to store all metadata with epoch as key
    mapping(uint64 => Metadata) metadata_set;

    // to identify current highest epoch number
    uint64 highest_epoch = U64_MAX;

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
        target.gas_limit = metadata.gas_limit;
        target.gas_price = metadata.gas_price;
        target.interval = metadata.interval;
        target.propose_ratio = metadata.propose_ratio;
        target.prevote_ratio = metadata.prevote_ratio;
        target.precommit_ratio = metadata.precommit_ratio;
        target.brake_ratio = metadata.brake_ratio;
        target.tx_num_limit = metadata.tx_num_limit;
        target.max_tx_size = metadata.max_tx_size;
        target.last_checkpoint_block_hash = metadata.last_checkpoint_block_hash;
        target.crossToken = metadata.crossToken;
        for (uint256 i = 0; i < metadata.verifier_list.length; i++) {
            target.verifier_list.push(metadata.verifier_list[i]);
        }
        highest_epoch = epoch;
    }

    // get metadata from `metadata_set` by epoch
    function getMetadata(uint64 epoch) public view returns (Metadata memory) {
        Metadata memory metadata = metadata_set[epoch];
        require(metadata.gas_limit != 0, "fatal/non-indexed epoch");
        return metadata;
    }

    function verifierList() external view returns (address[] memory, uint256) {
        uint256 length = metadata_set[highest_epoch].verifier_list.length;
        address[] memory verifiers = new address[](length);

        for (uint256 i = 0; i < length; ++i) {
            verifiers[i] = metadata_set[highest_epoch]
                .verifier_list[i]
                .address_;
        }

        return (verifiers, highest_epoch);
    }

    function isProposer(address verifier) external view returns (bool) {
        ValidatorExtend memory proposer;

        uint256 length = metadata_set[highest_epoch].verifier_list.length;
        for (uint256 i = 0; i < length; ++i) {
            if (
                metadata_set[highest_epoch].verifier_list[i].propose_weight >
                proposer.propose_weight
            ) {
                proposer = metadata_set[highest_epoch].verifier_list[i];
            }
        }

        return verifier == proposer.address_;
    }
}
