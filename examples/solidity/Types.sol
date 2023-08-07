// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0;

struct VerifyPayload {
    CellDep[]     cellDeps;
    HeaderDep[]   headerDeps;
    OutPoint[]    inputs;
    WitnessArgs[] witnesses;
}

struct OutPoint {
    bytes32 txHash;
    uint32  index;
}

struct CellDep {
    OutPoint outPoint;
    uint8    index;
}

struct HeaderDep {
    bytes32 headerHash;
}

struct Cell {
    CellOutput cellOutput;
    bytes      cellData;
    bool       isConsumed;
    uint64     createdNumber;
    uint64     consumedNumber;
}

struct CellOutput {
    uint64   capacity;
    Script   lock;
    Script[] type_;
}

struct Script {
    ScriptHashType hashType;
    bytes32        codeHash;
    bytes          args;
}

struct WitnessArgs {
    bytes lock;
    bytes inputType;
    bytes outputType;
}

enum ScriptHashType {
    Data,
    Type,
    Data1
}

struct Header {
    uint32  version;
    uint32  compactTarget;
    uint64  timestamp;
    uint64  number;
    uint64  epch;
    bytes32 parentHash;
    bytes32 transactionsRoot;
    bytes32 proposalsHash;
    bytes32 extraHash;
    bytes32 dao;
    uint128 nonce;
    bytes   extension;
    bytes32 blockHash;
}
