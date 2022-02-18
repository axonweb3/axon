let { expect, use } = require("chai")
let { deployContract, MockProvider, solidity } = require("ethereum-waffle")
let MetadataManager = require("../build/MetadataManager.json")

use(solidity)

function hexToBytes(hex) {
    for (var bytes = [], c = 0; c < hex.length; c += 2)
        bytes.push(parseInt(hex.substr(c, 2), 16));
    return bytes;
}

describe("MetadataManager", () => {
    const [wallet, walletTo] = new MockProvider().getWallets()
    let contract = deployContract(wallet, MetadataManager)
    let metadata = null

    beforeEach(async () => {
        metadata = {
            version: {
                start: 2,
                end: 3
            },
            epoch: 1,
            gas_limit: 1000,
            gas_price: 1000000,
            interval: 20,
            verifier_list: [
                {
                    bls_pub_key: hexToBytes("68656c6c6f20776f726c64"),
                    pub_key: hexToBytes("68656c6c6f20776f726c64"),
                    address_: wallet.address,
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
        await expect((await contract).getMetadata(1))
            .to.be.reverted
    })

    it("`appendMetadata` successed on epoch of 1 and on version of [2, 3]", async () => {
        await expect((await contract).appendMetadata(metadata))
            .to.be.not.reverted
    })

    it("`getMetadata` successed on epoch of 1", async () => {
        await expect((await contract).getMetadata(1))
            .to.be.not.reverted
    })

    it("`appendMetadata` failed on discontinuous epoch of 3", async () => {
        metadata.epoch = 3
        await expect((await contract).appendMetadata(metadata))
            .to.be.reverted
    })

    it("`appendMetadata` failed on discontinuous version of [5, 6]", async () => {
        metadata.version = { start: 5, end: 6 }
        await expect((await contract).appendMetadata(metadata))
            .to.be.reverted
    })

    it("`appendMetadata` failed on mismatched address", async () => {
        metadata.verifier_list[0].address_ = walletTo.address
        await expect((await contract).appendMetadata(metadata))
            .to.be.reverted
    })

    it("`appendMetadata` successed on epoch of 2 and on version of [4, 5]", async () => {
        metadata.epoch = 2
        metadata.version = { start: 4, end: 5 }
        await expect((await contract).appendMetadata(metadata))
            .to.be.not.reverted
    })
})
