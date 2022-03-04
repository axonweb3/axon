const { expect } = require("chai")
const { ethers } = require("hardhat")

function hexToBytes(hex) {
    for (var bytes = [], c = 0; c < hex.length; c += 2)
        bytes.push(parseInt(hex.substr(c, 2), 16));
    return bytes;
}

describe("CrossChain", () => {
    let wallets = null
    let contract = null
    let layer1asset = {
        udt_hash: hexToBytes("63b9fe46a9217a85203fc0cd7b67f3238ec93889d21fefb7cf11d40a1c3ddd9c"),
        cross_chain_tx_hash: hexToBytes("63b9fe46a9217a85203fc0cd7b67f3238ec93889d21fefb7cf11d40a1c3ddd9c"),
        erc20_address: ""
    }

    before(async () => {
        let deployer = await ethers.getContractFactory("CrossChain")
        contract = await deployer.deploy()
        await contract.deployed()
        wallets = await ethers.getSigners()
        layer1asset.erc20_address = wallets[0].address
    })

    it("should fail on inserting parameters with mismatched erc20_address", async () => {
        await expect(contract.insert(wallets[1].address, layer1asset))
            .to.be.revertedWith("fatal/mismatched erc20 address")
    })

    it("should succeed on inserting valid parameters", async () => {
        await expect(contract.insert(wallets[0].address, layer1asset))
            .to.be.not.reverted
    })

    it("should fail on getting asset by non-indexed erc20 address", async () => {
        await expect(contract.get(wallets[1].address))
            .to.be.revertedWith("fatal/non-indexed erc20 address")
    })

    it("should succeed on getting asset by indexed erc20 address", async () => {
        await expect(contract.get(wallets[0].address))
            .to.be.not.reverted
    })
})
