// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.0;

import "../libraries/CkbType.sol";

// **Notice**
// This file only defines the interface of CKB light client contract. The real
// implementation is in `core/executor/src/system_contract/ckb_light_client`.
interface CkbLightClient {
    function setState(bool allowRead) external;

    function update(CkbType.Header[] calldata headers) external;

    function rollback(bytes32[] calldata blockHashes) external;
}
