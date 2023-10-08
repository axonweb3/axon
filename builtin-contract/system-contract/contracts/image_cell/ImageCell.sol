// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

import "../libraries/CkbType.sol";

library ImageCellType {
    struct BlockUpdate {
        uint64 blockNumber;
        CkbType.OutPoint[] txInputs;
        CkbType.CellInfo[] txOutputs;
    }

    struct BlockRollBlack {
        CkbType.OutPoint[] txInputs;
        CkbType.OutPoint[] txOutputs;
    }
}

interface ImageCell {
    function setState(bool allowRead) public;

    function update(ImageCellType.BlockUpdate[] calldata blocks) public;

    function rollback(ImageCellType.BlockRollBlack[] calldata blocks) public;
}
