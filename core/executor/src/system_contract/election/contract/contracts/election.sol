// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

contract NodeElection {
    // Mapping of node IDs to their vote count
    mapping(bytes32 => uint) public votes;

    // Array of all node IDs that have been registered
    bytes32[] public nodes;

    // Event that is emitted whenever a node receives a vote
    event NodeVoted(bytes32 indexed node);

    // Function to register a new node
    function registerNode(bytes32 nodeId) public {
        // Add the node to the array of registered nodes
        nodes.push(nodeId);
    }

    // Function to cast a vote for a node
    function vote(bytes32 nodeId) public {
        // Increment the vote count for the given node
        votes[nodeId]++;

        // Emit an event to signal that the node has received a vote
        emit NodeVoted(nodeId);
    }

    // Function to get the vote count for a given node
    function getVoteCount(bytes32 nodeId) public view returns (uint) {
        return votes[nodeId];
    }

    // Function to get the total number of votes that have been cast
    function getTotalVoteCount() public view returns (uint) {
        uint count = 0;
        for (uint i = 0; i < nodes.length; i++) {
            count += votes[nodes[i]];
        }
        return count;
    }
}

