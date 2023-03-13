// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0;

import "../libraries/CkbType.sol";

contract CkbLightClient {
    using CkbType for CkbType.Header;
    
    function setState(bool allowRead) public view {}

    function update(CkbType.Header[] calldata headers) public view {}

    function rollback(bytes32[] calldata blockHashes) public view {}
}
