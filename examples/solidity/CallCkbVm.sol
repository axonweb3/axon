// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "./Types.sol";

contract CallCkbVm {
    event CallCkbVmEvent(int8);
    event NotGetCellEvent();

    int8 ret;

    function testCallCkbVm(
        bytes32 txHash,
        uint32 index,
        uint8 depType,
        bytes[] memory input_args
    ) public {
        OutPoint memory outPoint = OutPoint(txHash, index);
        (bool isSuccess, bytes memory res) = address(0x0104).staticcall(
            abi.encode(CellDep(outPoint, depType), input_args)
        );

        if (isSuccess) {
            ret = int8(uint8(res[0]));
            emit CallCkbVmEvent(ret);
        } else {
            emit NotGetCellEvent();
        }
    }

    function callCkbVm() public view returns (int8) {
        return ret;
    }
}
