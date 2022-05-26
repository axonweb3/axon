const { expect } = require("chai")
const { ethers } = require("hardhat")
const { deployMockContract } = require('@ethereum-waffle/mock-contract');

function hexToBytes(hex) {
    for (var bytes = [], c = 0; c < hex.length; c += 2)
        bytes.push(parseInt(hex.substr(c, 2), 16));
    return bytes;
}

async function deployMirrorToken(owner) {
    const MirrorToken = await ethers.getContractFactory('MirrorToken');
    contract = await MirrorToken.connect(owner).deploy("CKB", "CKB");

    return contract;
}

async function deployTestToken(owner) {
    const TestToken = await ethers.getContractFactory('TestToken')
    contract = await TestToken.connect(owner).deploy();

    return contract;
}

describe("CrossChain", () => {
    let owner = null;
    let wallets = null;
    let contract = null;
    let wckb = null;
    let mirrorToken = null;
    let simpleToken = null;
    let ATAddress = `0x${'0'.repeat(40)}`;
    let lockscript = ethers.utils.keccak256(ethers.utils.toUtf8Bytes('kk'));

    beforeEach(async () => {
        [owner, ...wallets] = await ethers.getSigners();

        let deployer = await ethers.getContractFactory("CrossChain");
        contract = await deployer.connect(owner).deploy([owner.address, wallets[0].address, wallets[1].address, wallets[2].address], 3, wallets[5].address, '', '');
        await contract.deployed();

        await contract.connect(owner).setWCKBMin(1);

        wckb = await deployMirrorToken(owner);
        await contract.connect(owner).setWCKB(wckb.address);
        await contract.connect(owner).addMirrorToken(wckb.address);
        await contract.connect(owner).setTokenConfig(wckb.address, [10, 100]);

        mirrorToken = await deployMirrorToken(owner);
        await contract.connect(owner).addMirrorToken(mirrorToken.address);
        await contract.connect(owner).setTokenConfig(mirrorToken.address, [10, 100]);

        simpleToken = await deployTestToken(owner);
        await contract.connect(owner).setTokenConfig(simpleToken.address, [10, 100]);

        await contract.connect(owner).setTokenConfig(ATAddress, [10, 100]);
    });

    it("user lock AT should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).lockAT(lockscript, { value: 100 }))
            .emit(contract, 'CrossToCKB')
            .withArgs(lockscript, ATAddress, 100, 1);

        expect(await wckb.balanceOf(contract.address)).equal(1);
        expect(await ethers.provider.getBalance(contract.address)).equal(100);
    });

    it("user lock AT should fail while value is zero", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.lockAT(lockscript)).revertedWith('CrossChain: value must be more than 0');
    });

    it("user lock AT should fail while the wckb is not enough", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).lockAT(lockscript, { value: 100 }))
            .revertedWith('ERC20: transfer amount exceeds balance');
    });

    it("user lock Token should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await simpleToken.connect(owner).transfer(wallet.address, 1000);
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await simpleToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, simpleToken.address, 100))
            .emit(contract, 'CrossToCKB')
            .withArgs(lockscript, simpleToken.address, 100, 1);

        expect(await wckb.balanceOf(contract.address)).equal(1);
        expect(await simpleToken.balanceOf(contract.address)).equal(100);
    });

    it("user lock Token should fail while the amount is 0", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await simpleToken.connect(owner).transfer(wallet.address, 1000);
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await simpleToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, simpleToken.address, 0))
            .revertedWith('CrossChain: amount must be more than 0');

    });

    it("user lock token should fail while amount is not enough", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await simpleToken.connect(owner).transfer(wallet.address, 10);
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await simpleToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, simpleToken.address, 100))
            .revertedWith('ERC20: transfer amount exceeds balance');
    });

    it("user lock Token should fail while wckb is not enough", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await simpleToken.connect(owner).transfer(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await simpleToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, simpleToken.address, 100))
            .revertedWith('CrossChain: amount of wckb is insufficient');
    });

    it("user burn ckb should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await wckb.connect(owner).transferOwnership(contract.address);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, wckb.address, 100))
            .emit(contract, 'CrossToCKB')
            .withArgs(lockscript, wckb.address, 99, 1);

        expect(await wckb.balanceOf(contract.address)).equal(1);
    });

    it("user burn ckb should fail while wckb is not enough", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 10);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await wckb.connect(owner).transferOwnership(contract.address);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, wckb.address, 100))
            .revertedWith('ERC20: burn amount exceeds balance');
    });

    it("user burn ckb should fail while wckb is not enough for extra amount", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 99);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await wckb.connect(owner).transferOwnership(contract.address);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, wckb.address, 100))
            .revertedWith('ERC20: transfer amount exceeds balance');
    });

    it("user burn ckb should fail while amount is zero", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await wckb.connect(owner).transferOwnership(contract.address);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, wckb.address, 0))
            .revertedWith('CrossChain: amount must be more than 0');
    });

    it("user burn mirror token should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });

        await wckb.connect(owner).mint(wallet.address, 1000);
        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await wckb.connect(owner).transferOwnership(contract.address);

        await mirrorToken.connect(owner).mint(wallet.address, 1000);
        await mirrorToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await mirrorToken.connect(owner).transferOwnership(contract.address);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, mirrorToken.address, 100))
            .emit(contract, 'CrossToCKB')
            .withArgs(lockscript, mirrorToken.address, 100, 1);

        expect(await wckb.balanceOf(contract.address)).equal(1);
        expect(await mirrorToken.balanceOf(wallet.address)).equal(900);
    });

    it("user burn mirror token should fail while amount is zero", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });

        await wckb.connect(owner).mint(wallet.address, 1000);
        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await wckb.connect(owner).transferOwnership(contract.address);

        await mirrorToken.connect(owner).mint(wallet.address, 1000);
        await mirrorToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await mirrorToken.connect(owner).transferOwnership(contract.address);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, mirrorToken.address, 0))
            .revertedWith('CrossChain: amount must be more than 0');
    });

    it("user burn mirror token should fail while ckb is not enough", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });

        await mirrorToken.connect(owner).mint(wallet.address, 1000);
        await mirrorToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await mirrorToken.connect(owner).transferOwnership(contract.address);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, mirrorToken.address, 100))
            .revertedWith('CrossChain: amount of wckb is insufficient');
    });

    it("user burn mirror token should fail while token is not enough", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });

        await wckb.connect(owner).mint(wallet.address, 1000);
        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await wckb.connect(owner).transferOwnership(contract.address);

        await mirrorToken.connect(owner).mint(wallet.address, 1000);
        await mirrorToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await mirrorToken.connect(owner).transferOwnership(contract.address);

        await expect(contract.connect(wallet).crossTokenToCKB(lockscript, mirrorToken.address, 100000))
            .revertedWith('ERC20: burn amount exceeds balance');
    });
});