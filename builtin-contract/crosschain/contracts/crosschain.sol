// SPDX-License-Identifier: GPL-3.0

pragma solidity >=0.7.0;

// import "hardhat/console.sol";

contract CrossChain {
    struct Layer1Asset {
        bytes32 udt_hash;
        bytes32 cross_chain_tx_hash;
        address erc20_address;
    }

    // to store all layer1 assets with erc20 as key
    mapping(address => Layer1Asset) assets;

    // push new layer1 asset into `assets`
    function insert(address erc20_address, Layer1Asset memory asset) public {
        require(
            erc20_address == asset.erc20_address, 
            "fatal/mismatched erc20 address"
        );
        assets[erc20_address] = asset;
    }

    // get layer1 asset from `assets` by erc20
    function get(address erc20_address) public view returns (Layer1Asset memory) {
        require(
            assets[erc20_address].erc20_address == erc20_address,
            "fatal/non-indexed erc20 address"
        );
        return assets[erc20_address];
    }
}
