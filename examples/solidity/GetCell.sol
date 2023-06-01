// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "./Types.sol";

contract GetCell {
    event GetCellEvent(Cell);
    event NotGetCellEvent();

    Cell cell;

    function testGetCell(bytes32 txHash, uint32 index) public {
        (bool isSuccess, bytes memory res) = address(0x0103).staticcall(abi.encode(OutPoint(txHash, index)));

        if (isSuccess) {
            cell = abi.decode(res, (Cell));
            emit GetCellEvent(cell);
        } else {
            emit NotGetCellEvent();
        }
    }

    function getCell() public view returns (Cell memory) {
        return cell;
    }
}
