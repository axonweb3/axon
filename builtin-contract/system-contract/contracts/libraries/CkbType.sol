// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

library CkbType {
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

    struct CellInfo {
        OutPoint outPoint;
        CellOutput output;
        bytes data;
    }

    struct OutPoint {
        bytes32 txHash;
        uint32 index;
    }

    struct CellOutput {
        uint64 capacity;
        Script lock;
        Script[] type_;
    }

    struct Script {
        bytes32 codeHash;
        ScriptHashType hashType;
        bytes args;
    }

    enum ScriptHashType {
        Data, // Type "data" matches script code via cell data hash, and run the script code in v0 CKB VM.
        Type, // Type "type" matches script code via cell type script hash.
        Data1 // Type "data1" matches script code via cell data hash, and run the script code in v1 CKB VM.
    }
}
