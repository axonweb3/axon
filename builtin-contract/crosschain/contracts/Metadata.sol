// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

interface IMetadata {
    function isProposer(address relayer) external returns (bool);

    function verifierList() external returns (address[] memory, uint256);

    function isVerifier(address relayer) external returns (bool);
}
