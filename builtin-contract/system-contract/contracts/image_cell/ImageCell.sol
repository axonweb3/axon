// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

import "../libraries/CkbType.sol";

library ImageCell {
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

// **Notice**
// This file only defines the interface of image cell contract. The real
// implementation is in `core/executor/src/system_contract/image_cell`.
interface ImageCellType {
    function setState(bool allowRead) external;

    function update(ImageCell.BlockUpdate[] calldata blocks) external;

    function rollback(ImageCell.BlockRollBlack[] calldata blocks) external;
}
