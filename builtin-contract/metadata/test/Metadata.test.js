const { expect } = require("chai")
const { ethers, upgrades } = require("hardhat")
const ABI = require('../artifacts/contracts/metadata.sol/MetadataManager.json');
const ERC1967Proxy = require('@openzeppelin/upgrades-core/artifacts/@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol/ERC1967Proxy.json');

function hexToBytes(hex) {
    for (var bytes = [], c = 0; c < hex.length; c += 2)
        bytes.push(parseInt(hex.substr(c, 2), 16));
    return bytes;
}

async function deployProxy(deployer, { params, initializer }) {
    // deploy proxy implementation
    const impl = await deployer.deploy();
    // console.log('metadata:', impl);
    await impl.deployed();
    // make proxy construction params
    const abi = new ethers.utils.Interface(ABI.abi);
    const data = abi.encodeFunctionData(initializer, params);
    // deploy proxy contract
    const proxy_deployer = await ethers.getContractFactory(ERC1967Proxy.abi, ERC1967Proxy.bytecode);
    const proxy = await proxy_deployer.deploy(impl.address, data);
    await proxy.deployed();
    // attach proxy contract
    return deployer.attach(proxy.address);
}

describe("Testing MetadataManager", () => {
    let wallets = null
    let contract = null
    let metadata = null

    before(async () => {
        let deployer = await ethers.getContractFactory("MetadataManager")
        // contract = await deployer.deploy()
        // contract = await upgrades.deployProxy(deployer, [], { initializer: 'construct' });
        // await contract.deployed()
        contract = await deployProxy(deployer, { params: [], initializer: 'construct' });
        wallets = await ethers.getSigners()
        metadata = {
            version: {
                start: 1,
                end: 2
            },
            epoch: 0,
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
            last_checkpoint_block_hash: hexToBytes("63b9fe46a9217a85203fc0cd7b67f3238ec93889d21fefb7cf11d40a1c3ddd9c"),
        }
    })

    it("`getMetadata` failed on non-indexed epoch of 0", async () => {
        await expect(contract.getMetadata(0))
            .to.be.revertedWith("fatal/non-indexed epoch")
    })

    it("`appendMetadata` successed on epoch of 0 and on version of [1, 2]", async () => {
        metadata.epoch = 0
        metadata.version = { start: 1, end: 2 }
        await expect(contract.appendMetadata(metadata))
            .to.be.not.reverted
    })

    it("`getMetadata` successed on epoch of 0", async () => {
        await expect(contract.getMetadata(0))
            .to.be.not.reverted
    })

    it("`appendMetadata` successed on epoch of 1 and on version of [3, 4]", async () => {
        metadata.epoch = 1
        metadata.version = { start: 3, end: 4 }
        await expect(contract.appendMetadata(metadata))
            .to.be.not.reverted
    })

    it("`appendMetadata` failed on mismatched address", async () => {
        metadata.epoch = 2
        metadata.version = { start: 5, end: 6 }
        metadata.verifier_list[0].address_ = wallets[1].address
        await expect(contract.appendMetadata(metadata))
            .to.be.revertedWith("fatal/verifier_list has no sender")
    })

    it("`appendMetadata` failed on discontinuous epoch of 3", async () => {
        metadata.epoch = 3
        metadata.version = { start: 5, end: 6 }
        metadata.verifier_list[0].address_ = wallets[0].address
        await expect(contract.appendMetadata(metadata))
            .to.be.revertedWith("fatal/discontinuous epoch")
    })

    it("`appendMetadata` failed on discontinuous version of [6, 7]", async () => {
        metadata.epoch = 2
        metadata.version = { start: 6, end: 7 }
        await expect(contract.appendMetadata(metadata))
            .to.be.revertedWith("fatal/discontinuous version")
    })

    it("`isProposer` should return false while address is not a proposer", async () => {
        expect(await contract.isProposer(wallets[10].address)).false;
    });

    it("`isProposer` should return true while address is a proposer", async () => {
        expect(await contract.isProposer(wallets[0].address)).true;
    });

    it("`isVerifier` should return false while address is not a verifier", async () => {
        expect(await contract.isVerifier(wallets[10].address)).false;
    });

    it("`isVerifier` should return true while address is a verifier", async () => {
        expect(await contract.isVerifier(wallets[0].address)).true;
    });

    // it("`appendMetadata` failed on block.number out of version of [5, 6]", async () => {
    //     metadata.epoch = 2
    //     metadata.version = { start: 5, end: 6 }
    //     await expect(contract.appendMetadata(metadata))
    //         .to.be.revertedWith("fatal/invalid version")
    // })
})
