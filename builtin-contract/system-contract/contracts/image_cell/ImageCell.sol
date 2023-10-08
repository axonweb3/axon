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

// **Notice**
// This file only defines the interface of image cell contract. The real
// implementation is in `core/executor/src/system_contract/image_cell`.
interface ImageCell {
    function setState(bool allowRead) external;

    function update(ImageCellType.BlockUpdate[] calldata blocks) external;

    function rollback(ImageCellType.BlockRollBlack[] calldata blocks) external;
}
