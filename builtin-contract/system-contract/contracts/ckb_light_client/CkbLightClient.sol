// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0;

import "../libraries/CkbType.sol";

interface CkbLightClient {
    function setState(bool allowRead) public;

    function update(CkbTypes.Header[] calldata headers) public;

    function rollback(bytes32[] calldata blockHashes) public;
}
