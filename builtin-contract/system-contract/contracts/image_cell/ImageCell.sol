// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

import "../libraries/CkbType.sol";

contract ImageCell {
    using CkbType for CkbType.CellInfo;
    using CkbType for CkbType.OutPoint;

    struct BlockUpdate {
        uint64 blockNumber;
        CkbType.OutPoint[] txInputs;
        CkbType.CellInfo[] txOutputs;
    }

    struct BlockRollBlack {
        CkbType.OutPoint[] txInputs;
        CkbType.OutPoint[] txOutputs;
    }

    function setState(bool allowRead) public view {}

    function update(BlockUpdate[] calldata blocks) public view {}

    function rollback(BlockRollBlack[] calldata blocks) public view {}
}
