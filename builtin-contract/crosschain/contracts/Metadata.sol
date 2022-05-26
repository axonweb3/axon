// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.8.0;

interface IMetadata {
    function isProposer(address relayer) external returns (bool);
}
