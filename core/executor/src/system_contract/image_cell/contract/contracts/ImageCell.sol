// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

import "./libraries/CkbType.sol";

contract ImageCell {
    using CkbType for CkbType.CellInfo;
    using CkbType for CkbType.OutPoint;

    function setState(bool allowRead) public view {}

    function update(
        uint64 blockNumber,
        CkbType.OutPoint[] calldata inputs,
        CkbType.CellInfo[] calldata outputs
    ) public view {}

    function rollback(
        CkbType.OutPoint[] calldata inputs,
        CkbType.OutPoint[] calldata outputs
    ) public view {}
}
