// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "./Types.sol";

contract VerifyByCkb {
    event VerifyByCkbErent(uint64);
    event NotGetCellEvent();

    uint64 ret;

    function testVerifyByCkb(
        CellDep[] memory cellDeps,
        HeaderDep[] memory HeaderDeps,
        OutPoint[] memory inputs,
        WitnessArgs[] memory witnesses
    ) public {
        VerifyPayload memory payload = VerifyPayload(
            cellDeps,
            HeaderDeps,
            inputs,
            witnesses
        );
        (bool isSuccess, bytes memory res) = address(0x0105).staticcall(
            abi.encode(payload)
        );

        if (isSuccess) {
            ret = bytesToUint64(res);
            emit VerifyByCkbErent(ret);
        } else {
            emit NotGetCellEvent();
        }
    }

    function callCkbVm() public view returns (uint64) {
        return ret;
    }

    function bytesToUint64(bytes memory b) public pure returns (uint64) {
        uint64 number;

        for (uint i = 0; i < 8; i++) {
            number = number + uint8(b[i]);
        }

        return number;
    }
}
