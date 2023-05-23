// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import "./Types.sol";

contract GetHeader {
    event GetHeaderEvent(Header);
    event NotGetHeaderEvent();

    Header header;

    function testGetHeader(bytes32 blockHash) public {
        (bool isSuccess, bytes memory res) = address(0x0102).staticcall(abi.encode(blockHash));

        if (isSuccess) {
            header = abi.decode(res, (Header));
            emit GetHeaderEvent(header);
        } else {
            emit NotGetHeaderEvent();
        }
    }

    function getHeader() public view returns (Header memory) {
        return header;
    }
}
