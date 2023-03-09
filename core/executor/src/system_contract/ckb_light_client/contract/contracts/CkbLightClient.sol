// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

contract CkbLightClient {
    struct Header {
        uint32 version;
        uint32 compactTarget;
        uint64 timestamp;
        uint64 number;
        uint64 epoch;
        bytes32 parentHash;
        bytes32 transactionsRoot;
        bytes32 proposalsHash;
        bytes32 unclesHash;
        bytes32 dao;
        uint128 nonce;
        bytes32 blockHash;
    }

    function setState(bool allowRead) public view {}

    function update(Header calldata header) public view {}

    function rollback(
        bytes32 blockHash,
        uint64 blockNumber
    ) public view {}
}
