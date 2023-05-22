// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "../IPrecompiles.sol";

contract GetCell {
    event GetCellEvent(Cell);
    event NotGetCellEvent();

    Cell cell;

    function testGetCell(bytes32 txHash, uint32 index) public {
        cell = CellProvider(address(0x0103)).getCell(txHash, index);
        if (cell.exists) {
            emit GetCellEvent(cell);
        } else {
            emit NotGetCellEvent();
        }
    }

    function getCell() public view returns (Cell memory) {
        return cell;
    }
}
