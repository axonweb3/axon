const { expect } = require("chai")
const { ethers } = require("hardhat")
const { deployMockContract } = require('@ethereum-waffle/mock-contract');
const { abi } = require('../artifacts/contracts/Metadata.sol/IMetadata.json');

function hexToBytes(hex) {
    for (var bytes = [], c = 0; c < hex.length; c += 2)
        bytes.push(parseInt(hex.substr(c, 2), 16));
    return bytes;
}

function recordsHash(records) {
    return ethers.utils.keccak256(
        ethers.utils.defaultAbiCoder.encode(
            ["tuple(address to, address tokenAddress, uint256 sUDTAmount, uint256 CKBAmount, bytes32 txHash)[]"],
            [records],
        ),
    );
}

async function deployMirrorToken(owner) {
    const MirrorToken = await ethers.getContractFactory('MirrorToken');
    contract = await MirrorToken.connect(owner).deploy("CKB", "CKB", 8);

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
    let domain = null;
    let metadata = null;
    let ATAddress = `0x${'0'.repeat(40)}`;
    let lockscript = ethers.utils.keccak256(ethers.utils.toUtf8Bytes('kk'));
    const crossFromCKBTypes = {
        Transaction: [
            { name: "recordsHash", type: "bytes32" },
            { name: "nonce", type: "uint256" }
        ],
    };


    beforeEach(async () => {
        [owner, ...wallets] = await ethers.getSigners();

        metadata = await deployMockContract(owner, abi);
        await metadata.mock.verifierList.returns(
            [owner.address, wallets[0].address, wallets[1].address, wallets[2].address],
            1,
        );
        await metadata.mock.isVerifier.returns(true);

        wckb = await deployMirrorToken(owner);

        let deployer = await ethers.getContractFactory("CrossChain");
        contract = await deployer.connect(owner).deploy(metadata.address, wckb.address);
        await contract.deployed();

        await contract.connect(owner).setTokenConfig(wckb.address, [10, 10000000]);
        await wckb.connect(owner).grantRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes('MANAGER_ROLE')), owner.address);
        await contract.connect(owner).setWCKBMin(1);

        await wckb.connect(owner).grantRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes('MANAGER_ROLE')), contract.address);
        await wckb.connect(owner).grantRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes('MANAGER_ROLE')), contract.address);

        mirrorToken = await deployMirrorToken(owner);
        await contract.connect(owner).addMirrorToken(mirrorToken.address, lockscript);
        await contract.connect(owner).setTokenConfig(mirrorToken.address, [10, 10000000]);
        await mirrorToken.connect(owner).grantRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes('MANAGER_ROLE')), owner.address);
        await mirrorToken.connect(owner).grantRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes('MANAGER_ROLE')), contract.address);
        await mirrorToken.connect(owner).grantRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes('MANAGER_ROLE')), contract.address);

        simpleToken = await deployTestToken(owner);
        await contract.connect(owner).setTokenConfig(simpleToken.address, [10, 10000000]);

        await contract.connect(owner).setTokenConfig(ATAddress, [10, 200]);

        await contract.connect(owner).addWhitelist(wckb.address);

        await contract.connect(owner).addWhitelist(mirrorToken.address);

        await contract.connect(owner).addWhitelist(ATAddress);

        await contract.connect(owner).addWhitelist(simpleToken.address);

        domain = {
            name: 'test',
            version: '1',
            chainId: (await ethers.provider.getNetwork()).chainId,
            verifyingContract: contract.address
        };
    });

    it("user lock AT should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).lockAT("ckbAddress", { value: 100 }))
            .emit(contract, 'CrossToCKB')
            .withArgs("ckbAddress", ATAddress, 100, 1);

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

        await expect(contract.lockAT("ckbAddress")).revertedWith('CrossChain: value must be more than 0');
    });

    it("user lock AT should fail while the wckb is not enough", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).lockAT("ckbAddress", { value: 100 }))
            .revertedWith('ERC20: transfer amount exceeds balance');
    });

    it("user lock AT should alert while the amount is exceed the threshold", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).lockAT("ckbAddress", { value: 200 }))
            .emit(contract, 'CrossToCKBAlert')
            .withArgs("ckbAddress", ATAddress, 200, 1);

        expect(await wckb.balanceOf(contract.address)).equal(1);
        expect(await ethers.provider.getBalance(contract.address)).equal(200);

        let limitTxes = await contract.limitTxes();
        expect(limitTxes).length(1);
        expect(limitTxes[0].to).equal("ckbAddress");
        expect(limitTxes[0].tokenAddress).equal(ATAddress);
        expect(limitTxes[0].amount).equal(200);
        expect(limitTxes[0].minWCKBAmount).equal(1);

        await expect(contract.connect(owner).removeLimitTx(limitTxes[0])).not.reverted;
        limitTxes = await contract.limitTxes();
        expect(limitTxes).length(0);
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

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", simpleToken.address, 100))
            .emit(contract, 'CrossToCKB')
            .withArgs("ckbAddress", simpleToken.address, 100, 1);

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

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", simpleToken.address, 0))
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

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", simpleToken.address, 100))
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

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", simpleToken.address, 100))
            .revertedWith('CrossChain: amount of wckb is insufficient');
    });

    it("user lock Token should alert while it is exceed the threshold", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await simpleToken.connect(owner).transfer(wallet.address, 100000001);
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);
        await simpleToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", simpleToken.address, 10000001))
            .emit(contract, 'CrossToCKBAlert')
            .withArgs("ckbAddress", simpleToken.address, 10000001, 1);

        expect(await wckb.balanceOf(contract.address)).equal(1);
        expect(await simpleToken.balanceOf(contract.address)).equal(10000001);

        let limitTxes = await contract.limitTxes();
        expect(limitTxes).length(1);
        expect(limitTxes[0].to).equal("ckbAddress");
        expect(limitTxes[0].tokenAddress).equal(simpleToken.address);
        expect(limitTxes[0].amount).equal(10000001);
        expect(limitTxes[0].minWCKBAmount).equal(1);

        await expect(contract.connect(owner).removeLimitTx(limitTxes[0])).not.reverted;
        limitTxes = await contract.limitTxes();
        expect(limitTxes).length(0);
    });

    it("user burn ckb should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 1000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", wckb.address, 100))
            .emit(contract, 'CrossToCKB')
            .withArgs("ckbAddress", wckb.address, 99, 1);

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

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", wckb.address, 100))
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

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", wckb.address, 100))
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

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", wckb.address, 0))
            .revertedWith('CrossChain: amount must be more than 0');
    });

    it("user burn ckb should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await wckb.connect(owner).mint(wallet.address, 100000000);

        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", wckb.address, 20000000))
            .emit(contract, 'CrossToCKBAlert')
            .withArgs("ckbAddress", wckb.address, 19999999, 1);

        expect(await wckb.balanceOf(contract.address)).equal(1);
        expect(await wckb.balanceOf(wallet.address)).equal(80000000);

        let limitTxes = await contract.limitTxes();
        expect(limitTxes).length(1);
        expect(limitTxes[0].to).equal("ckbAddress");
        expect(limitTxes[0].tokenAddress).equal(wckb.address);
        expect(limitTxes[0].amount).equal(19999999);
        expect(limitTxes[0].minWCKBAmount).equal(1);

        await expect(contract.connect(owner).removeLimitTx(limitTxes[0])).not.reverted;
        limitTxes = await contract.limitTxes();
        expect(limitTxes).length(0);
    });

    it("user burn mirror token should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });

        await wckb.connect(owner).mint(wallet.address, 1000);
        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await mirrorToken.connect(owner).mint(wallet.address, 1000);
        await mirrorToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", mirrorToken.address, 100))
            .emit(contract, 'CrossToCKB')
            .withArgs("ckbAddress", mirrorToken.address, 100, 1);

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

        await mirrorToken.connect(owner).mint(wallet.address, 1000);
        await mirrorToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", mirrorToken.address, 0))
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

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", mirrorToken.address, 100))
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

        await mirrorToken.connect(owner).mint(wallet.address, 1000);
        await mirrorToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", mirrorToken.address, 100000))
            .revertedWith('ERC20: burn amount exceeds balance');
    });

    it("user burn mirror token should success", async () => {
        const wallet = ethers.Wallet.createRandom().connect(ethers.provider);
        await owner.sendTransaction({
            to: wallet.address,
            value: ethers.utils.parseEther('1.0'),
        });

        await wckb.connect(owner).mint(wallet.address, 1000);
        await wckb.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await mirrorToken.connect(owner).mint(wallet.address, 100000000);
        await mirrorToken.connect(wallet).approve(contract.address, ethers.constants.MaxUint256);

        await expect(contract.connect(wallet).crossTokenToCKB("ckbAddress", mirrorToken.address, 20000000))
            .emit(contract, 'CrossToCKBAlert')
            .withArgs("ckbAddress", mirrorToken.address, 20000000, 1);

        expect(await wckb.balanceOf(contract.address)).equal(1);
        expect(await mirrorToken.balanceOf(wallet.address)).equal(80000000);

        let limitTxes = await contract.limitTxes();
        expect(limitTxes).length(1);
        expect(limitTxes[0].to).equal("ckbAddress");
        expect(limitTxes[0].tokenAddress).equal(mirrorToken.address);
        expect(limitTxes[0].amount).equal(20000000);
        expect(limitTxes[0].minWCKBAmount).equal(1);

        await expect(contract.connect(owner).removeLimitTx(limitTxes[0])).not.reverted;
        limitTxes = await contract.limitTxes();
        expect(limitTxes).length(0);
    });

    it("cross wckb and sudt should success", async () => {
        const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
        const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

        const records = [
            {
                to: wallet1.address,
                tokenAddress: mirrorToken.address,
                sUDTAmount: 10,
                CKBAmount: 1000,
                txHash: lockscript,
                limitSign: 0,
            },
            {
                to: wallet2.address,
                tokenAddress: mirrorToken.address,
                sUDTAmount: 100,
                CKBAmount: 100000,
                txHash: lockscript,
                limitSign: 0,
            },
        ];

        const nonce = 0;

        const value = {
            recordsHash: recordsHash(records),
            nonce: nonce,
        };

        let signatures = '';

        await metadata.mock.isVerifier.returns(true);

        signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures += (await wallets[2]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures = '0x' + signatures;

        await contract.crossFromCKB(records, nonce);

        expect(await mirrorToken.balanceOf(wallet1.address)).equal(10);
        expect(await mirrorToken.balanceOf(wallet2.address)).equal(100);
        expect(await wckb.balanceOf(wallet1.address)).equal(1000);
        expect(await wckb.balanceOf(wallet2.address)).equal(100000);
    });

    it("cross wckb and sudt should fail while sender is not proposer", async () => {
        const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
        const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

        const records = [
            {
                to: wallet1.address,
                tokenAddress: mirrorToken.address,
                sUDTAmount: 10,
                CKBAmount: 1000,
                txHash: lockscript,
                limitSign: 0,
            },
            {
                to: wallet2.address,
                tokenAddress: mirrorToken.address,
                sUDTAmount: 100,
                CKBAmount: 100000,
                txHash: lockscript,
                limitSign: 0,
            },
        ];

        const nonce = 0;

        const value = {
            recordsHash: recordsHash(records),
            nonce: nonce,
        };

        let signatures = '';

        await metadata.mock.isVerifier.returns(false);

        signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures += (await wallets[2]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures = '0x' + signatures;

        await expect(contract.crossFromCKB(records, nonce))
            .to
            .revertedWith('CrossChain: sender must be verifier');
    });

    // it("cross wckb and sudt should fail while signatures are not enough", async () => {
    //     const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
    //     const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

    //     const records = [
    //         {
    //             to: wallet1.address,
    //             tokenAddress: mirrorToken.address,
    //             sUDTAmount: 10,
    //             CKBAmount: 1000,
    //             txHash: lockscript,
    //         },
    //         {
    //             to: wallet2.address,
    //             tokenAddress: mirrorToken.address,
    //             sUDTAmount: 100,
    //             CKBAmount: 100000,
    //             txHash: lockscript,
    //         },
    //     ];

    //     const nonce = await contract.crossFromCKBNonce();

    //     const value = {
    //         recordsHash: recordsHash(records),
    //         nonce: nonce,
    //     };

    //     let signatures = '';

    //     await metadata.mock.isProposer.returns(true);

    //     signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures = '0x' + signatures;

    //     await expect(contract.crossFromCKB(records, signatures, nonce)).to.revertedWith('CrossChain: signatures are not enough');
    // });

    // it("cross wckb and sudt should fail while valid signatures are not enough", async () => {
    //     const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
    //     const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

    //     const records = [
    //         {
    //             to: wallet1.address,
    //             tokenAddress: mirrorToken.address,
    //             sUDTAmount: 10,
    //             CKBAmount: 1000,
    //             txHash: lockscript,
    //         },
    //         {
    //             to: wallet2.address,
    //             tokenAddress: mirrorToken.address,
    //             sUDTAmount: 100,
    //             CKBAmount: 100000,
    //             txHash: lockscript,
    //         },
    //     ];

    //     const nonce = await contract.crossFromCKBNonce();

    //     const value = {
    //         recordsHash: recordsHash(records),
    //         nonce: nonce,
    //     };

    //     let signatures = '';

    //     await metadata.mock.isProposer.returns(true);

    //     signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[10]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures = '0x' + signatures;

    //     await expect(contract.crossFromCKB(records, signatures, nonce)).to.revertedWith('CrossChain: valid signatures are not enough');
    // });

    // it("cross wckb and sudt should fail while valid different signatures are not enough", async () => {
    //     const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
    //     const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

    //     const records = [
    //         {
    //             to: wallet1.address,
    //             tokenAddress: mirrorToken.address,
    //             sUDTAmount: 10,
    //             CKBAmount: 1000,
    //             txHash: lockscript,
    //         },
    //         {
    //             to: wallet2.address,
    //             tokenAddress: mirrorToken.address,
    //             sUDTAmount: 100,
    //             CKBAmount: 100000,
    //             txHash: lockscript,
    //         },
    //     ];

    //     const nonce = await contract.crossFromCKBNonce();

    //     const value = {
    //         recordsHash: recordsHash(records),
    //         nonce: nonce,
    //     };

    //     let signatures = '';

    //     await metadata.mock.isProposer.returns(true);

    //     signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures = '0x' + signatures;

    //     await expect(contract.crossFromCKB(records, signatures, nonce)).to.revertedWith('CrossChain: valid signatures are not enough');
    // });

    // it("cross wckb and sudt should alert while the amount exceed the threshold", async () => {
    //     const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
    //     const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

    //     const records = [
    //         {
    //             to: wallet1.address,
    //             tokenAddress: mirrorToken.address,
    //             amount: 1000000000,
    //             CKBAmount: 1000,
    //             txHash: ethers.utils.keccak256(ethers.utils.toUtf8Bytes('1')),
    //             retry: 0,
    //         },
    //         {
    //             to: wallet2.address,
    //             tokenAddress: mirrorToken.address,
    //             amount: 100,
    //             CKBAmount: 1000000000000,
    //             txHash: ethers.utils.keccak256(ethers.utils.toUtf8Bytes('2')),
    //             retry: 0,
    //         },
    //     ];

    //     let nonce = await contract.crossFromCKBNonce();

    //     const value = {
    //         recordsHash: recordsHash(records),
    //         nonce: nonce,
    //     };

    //     let signatures = '';

    //     await metadata.mock.isProposer.returns(true);

    //     signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[2]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures = '0x' + signatures;

    //     await expect(contract.crossFromCKB(records, signatures, nonce))
    //         .emit(contract, 'CrossFromCKBAlert').withArgs(wallet1.address, mirrorToken.address, 1000000000)
    //         .emit(contract, 'CrossFromCKBAlert').withArgs(wallet2.address, wckb.address, 1000000000000);

    //     nonce++;

    //     const limitTxes = await contract.limitTxes();
    //     expect(limitTxes).length(2);
    //     expect(limitTxes[0].txHash).equal(records[0].txHash);
    //     expect(limitTxes[1].txHash).equal(records[1].txHash);

    //     records[0].retry = 1;
    //     records[1].retry = 1;

    //     value.recordsHash = recordsHash(records);
    //     value.nonce = nonce;

    //     signatures = '';
    //     signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[2]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures = '0x' + signatures;

    //     await expect(contract.crossFromCKB(records, signatures, nonce)).not.reverted;

    //     expect(await wckb.balanceOf(wallet1.address)).equal(1000);
    //     expect(await mirrorToken.balanceOf(wallet1.address)).equal(1000000000);
    //     expect(await wckb.balanceOf(wallet2.address)).equal(1000000000000);
    //     expect(await mirrorToken.balanceOf(wallet2.address)).equal(100);
    // });

    // it("cross wckb and sudt should fail while nonce is not valid", async () => {
    //     const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
    //     const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

    //     const records = [
    //         {
    //             to: wallet1.address,
    //             tokenAddress: mirrorToken.address,
    //             sUDTAmount: 10,
    //             CKBAmount: 1000,
    //             txHash: lockscript,
    //         },
    //         {
    //             to: wallet2.address,
    //             tokenAddress: mirrorToken.address,
    //             sUDTAmount: 100,
    //             CKBAmount: 100000,
    //             txHash: lockscript,
    //         },
    //     ];

    //     const nonce = 10;

    //     const value = {
    //         recordsHash: recordsHash(records),
    //         nonce: nonce,
    //     };

    //     let signatures = '';

    //     await metadata.mock.isVerifier.returns(true);

    //     signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures += (await wallets[2]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
    //     signatures = '0x' + signatures;

    //     await expect(contract.crossFromCKB(records, nonce))
    //         .revertedWith('CrossChain: invalid nonce');
    // });

    it("cross wckb and sudt should success", async () => {
        const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
        const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

        await owner.sendTransaction({
            to: wallet1.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await simpleToken.connect(owner).transfer(wallet1.address, 1000);
        await wckb.connect(owner).mint(wallet1.address, 1000);
        await wckb.connect(wallet1).approve(contract.address, ethers.constants.MaxUint256);
        await simpleToken.connect(wallet1).approve(contract.address, ethers.constants.MaxUint256);

        await contract.connect(wallet1).crossTokenToCKB("ckbAddress", simpleToken.address, 100);

        const records = [
            {
                to: wallet1.address,
                tokenAddress: simpleToken.address,
                sUDTAmount: 10,
                CKBAmount: 1000,
                txHash: lockscript,
                limitSign: 0,
            },
            {
                to: wallet2.address,
                tokenAddress: mirrorToken.address,
                sUDTAmount: 100,
                CKBAmount: 100000,
                txHash: lockscript,
                limitSign: 0,
            },
        ];

        const nonce = 0;

        const value = {
            recordsHash: recordsHash(records),
            nonce: nonce,
        };

        let signatures = '';

        await metadata.mock.isVerifier.returns(true);

        signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures += (await wallets[2]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures = '0x' + signatures;

        await contract.crossFromCKB(records, nonce);

        expect(await simpleToken.balanceOf(wallet1.address)).equal(910);
        expect(await mirrorToken.balanceOf(wallet2.address)).equal(100);
        expect(await wckb.balanceOf(wallet2.address)).equal(100000);
    });

    it("cross wckb and sudt should success", async () => {
        const wallet1 = ethers.Wallet.createRandom().connect(ethers.provider);
        const wallet2 = ethers.Wallet.createRandom().connect(ethers.provider);

        await owner.sendTransaction({
            to: wallet1.address,
            value: ethers.utils.parseEther('1.0'),
        });
        await simpleToken.connect(owner).transfer(wallet1.address, 1000);
        await wckb.connect(owner).mint(wallet1.address, 1000);
        await wckb.connect(wallet1).approve(contract.address, ethers.constants.MaxUint256);

        await contract.connect(wallet1).lockAT("ckbAddress", { value: 100 });
        const balance = await ethers.provider.getBalance(wallet1.address);

        const records = [
            {
                to: wallet1.address,
                tokenAddress: ATAddress,
                sUDTAmount: 10,
                CKBAmount: 1000,
                txHash: lockscript,
                limitSign: 0,
            },
            {
                to: wallet2.address,
                tokenAddress: mirrorToken.address,
                sUDTAmount: 100,
                CKBAmount: 100000,
                txHash: lockscript,
                limitSign: 0,
            },
        ];

        const nonce = 0;

        const value = {
            recordsHash: recordsHash(records),
            nonce: nonce,
        };

        let signatures = '';

        await metadata.mock.isVerifier.returns(true);

        signatures += (await wallets[0]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures += (await wallets[1]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures += (await wallets[2]._signTypedData(domain, crossFromCKBTypes, value)).substring(2);
        signatures = '0x' + signatures;

        await contract.crossFromCKB(records, nonce);

        expect(await ethers.provider.getBalance(wallet1.address)).equal(balance.add(10));
        expect(await mirrorToken.balanceOf(wallet2.address)).equal(100);
        expect(await wckb.balanceOf(wallet2.address)).equal(100000);
    });
});