// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0;

struct OutPoint {
    bytes32 txHash;
    uint32 index;
}

struct Cell {
    CellOutput cellOutput;
    bytes cellData;
    bool isConsumed;
    uint64 createdNumber;
    uint64 consumedNumber;
}

struct CellOutput {
    uint64 capacity;
    Script lock;
    Script type_;
}

struct Script {
    ScriptHashType hashType;
    bytes32 codeHash;
    bytes args;
}

enum ScriptHashType {
    Data,
    Type,
    Data1
}
