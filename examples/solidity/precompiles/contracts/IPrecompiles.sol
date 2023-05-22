// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0;

/**
 * The address 0x0103 implements the function of getting cell.
 * `CellProvider` can be renamed, but `getCell` cannot.
 * Usage: Cell memory cell = CellProvider(address(0x0103)).getCell(txHash, index);
 */
interface CellProvider {
    function getCell(bytes32 txHash, uint32 index) external returns (Cell memory cell);
}

struct Cell {
    bool exists;
    bool hasTypeScript;
    bool hasConsumedNumber;
    uint64 createdNumber;
    uint64 consumedNumber;
    uint64 capacity;
    Script lock;
    Script type_;
    bytes data;
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
