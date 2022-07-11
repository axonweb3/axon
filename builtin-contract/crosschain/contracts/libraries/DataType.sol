// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

library DataType {
    struct TokenConfig {
        uint256 feeRatio;
        uint256 threshold;
    }

    struct CKBToAxonRecord {
        address to;
        address tokenAddress;
        uint256 sUDTAmount;
        uint256 CKBAmount;
        bytes32 txHash;
    }

    struct AxonToCKBRecord {
        address tokenAddress;
        uint256 amount;
        uint256 minWCKBAmount;
        string to;
        uint256 limitSign;
    }
}
