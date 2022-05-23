const { expect } = require("chai");
const { intToBuffer } = require("ethereumjs-util");
const { ethers } = require("hardhat")

async function deployMirrorToken(owner) {
  const MirrorToken = await ethers.getContractFactory('MirrorToken');
  const mirrorToken = await MirrorToken.connect(owner).deploy('testName', 'testSymbol');

  return mirrorToken;
}

describe("MirrorToken", () => {
  it("deploy a mirror token.", async () => {
    const [owner] = await ethers.getSigners();

    const mirrorToken = await deployMirrorToken(owner);
    expect(await mirrorToken.name()).equal('testName');
    expect(await mirrorToken.symbol()).equal('testSymbol');
  });

  it("only owner can mint new tokens.", async () => {
    const [owner, ...signers] = await ethers.getSigners();

    const token = await deployMirrorToken(owner);

    token.connect(owner).mint(signers[1].address, 10);
    expect(await token.connect(signers[1]).balanceOf(signers[1].address)).equal(10);

    await expect(token.connect(signers[2]).mint(signers[3].address, 10)).reverted;
    expect(await token.connect(owner).balanceOf(signers[3].address)).equal(0);
  });

  it("only owner can burn other account's tokens.", async () => {
    const [owner, ...signers] = await ethers.getSigners();

    const token = await deployMirrorToken(owner);

    token.connect(owner).mint(signers[1].address, 10);
    token.connect(owner).burn(signers[1].address, 1);
    expect(await token.connect(owner).balanceOf(signers[1].address)).equal(9);

    await expect(token.connect(signers[10]).burn(signers[1].address, 5)).reverted;
    expect(await token.connect(owner).balanceOf(signers[1].address)).equal(9);
  });
});