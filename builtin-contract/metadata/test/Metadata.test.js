const { expect } = require("chai")
const { ethers } = require("hardhat")

function hexToBytes(hex) {
    for (var bytes = [], c = 0; c < hex.length; c += 2)
        bytes.push(parseInt(hex.substr(c, 2), 16));
    return bytes;
}

describe("Testing MetadataManager", () => {
    let wallets = null
    let contract = null
    let metadata = null

    beforeEach(async () => {
        if (contract == null) {
            let deployer = await ethers.getContractFactory("MetadataManager")
            contract = await deployer.deploy()
            await contract.deployed()
        }
        wallets = await ethers.getSigners()
        metadata = {
            version: {
                start: 1,
                end: 2
            },
            epoch: 1,
            gas_limit: 1000,
            gas_price: 1000000,
            interval: 20,
            verifier_list: [
                {
                    bls_pub_key: hexToBytes("68656c6c6f20776f726c64"),
                    pub_key: hexToBytes("68656c6c6f20776f726c64"),
                    address_: wallets[0].address,
                    propose_weight: 3,
                    vote_weight: 10
                }
            ],
            propose_ratio: 23,
            prevote_ratio: 135,
            precommit_ratio: 8,
            brake_ratio: 1008,
            tx_num_limit: 55,
            max_tx_size: 400000,
            last_checkpoint_block_hash: hexToBytes("63b9fe46a9217a85203fc0cd7b67f3238ec93889d21fefb7cf11d40a1c3ddd9c")
        }
    })

    it("`getMetadata` failed on non-indexed epoch of 1", async () => {
        await expect(contract.getMetadata(1))
            .to.be.revertedWith("fatal/non-indexed epoch")
    })

    it("`appendMetadata` successed on epoch of 1 and on version of [1, 2]", async () => {
        await expect(contract.appendMetadata(metadata))
            .to.be.not.reverted
    })

    it("`getMetadata` successed on epoch of 1", async () => {
        await expect(contract.getMetadata(1))
            .to.be.not.reverted
    })


    it("`appendMetadata` successed on epoch of 2 and on version of [3, 4]", async () => {
        metadata.epoch = 2
        metadata.version = { start: 3, end: 4 }
        await expect(contract.appendMetadata(metadata))
            .to.be.not.reverted
    })

    it("`appendMetadata` failed on mismatched address", async () => {
        metadata.epoch = 3
        metadata.version = { start: 5, end: 6 }
        metadata.verifier_list[0].address_ = wallets[1].address
        await expect(contract.appendMetadata(metadata))
            .to.be.revertedWith("fatal/verifier_list has no sender")
    })

    it("`appendMetadata` failed on discontinuous epoch of 4", async () => {
        metadata.epoch = 4
        metadata.version = { start: 5, end: 6 }
        await expect(contract.appendMetadata(metadata))
            .to.be.revertedWith("fatal/discontinuous epoch")
    })

    it("`appendMetadata` failed on discontinuous version of [6, 7]", async () => {
        metadata.epoch = 3
        metadata.version = { start: 6, end: 7 }
        await expect(contract.appendMetadata(metadata))
            .to.be.revertedWith("fatal/discontinuous version")
    })

    it("`appendMetadata` failed on block.number out of version of [5, 6]", async () => {
        metadata.epoch = 3
        metadata.version = { start: 5, end: 6 }
        await expect(contract.appendMetadata(metadata))
            .to.be.revertedWith("fatal/invalid version")
    })
})
