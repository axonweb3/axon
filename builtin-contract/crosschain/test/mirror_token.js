const { expect } = require("chai");
const { intToBuffer } = require("ethereumjs-util");
const { ethers } = require("hardhat")

async function deployMirrorToken(owner, minter, burner) {
  const MirrorToken = await ethers.getContractFactory('MirrorToken');
  const mirrorToken = await MirrorToken.connect(owner).deploy('testName', 'testSymbol');

  await mirrorToken.connect(owner).grantRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes('MANAGER_ROLE')), minter.address);
  await mirrorToken.connect(owner).grantRole(ethers.utils.keccak256(ethers.utils.toUtf8Bytes('MANAGER_ROLE')), burner.address);

  return mirrorToken;
}

describe("MirrorToken", () => {
  it("deploy a mirror token.", async () => {
    const [owner] = await ethers.getSigners();

    const mirrorToken = await deployMirrorToken(owner, owner, owner);
    expect(await mirrorToken.name()).equal('testName');
    expect(await mirrorToken.symbol()).equal('testSymbol');
  });

  it("only who has minter role can mint new tokens.", async () => {
    const [owner, minter, burner, ...signers] = await ethers.getSigners();

    const token = await deployMirrorToken(owner, minter, burner);

    token.connect(minter).mint(signers[1].address, 10);
    expect(await token.connect(signers[1]).balanceOf(signers[1].address)).equal(10);

    await expect(token.connect(signers[2]).mint(signers[3].address, 10)).reverted;
    expect(await token.connect(owner).balanceOf(signers[3].address)).equal(0);
  });

  it("only who has burner role can burn other account's tokens.", async () => {
    const [owner, minter, burner, ...signers] = await ethers.getSigners();

    const token = await deployMirrorToken(owner, minter, burner);

    token.connect(minter).mint(signers[1].address, 10);
    token.connect(burner).burn(signers[1].address, 1);
    expect(await token.connect(owner).balanceOf(signers[1].address)).equal(9);

    await expect(token.connect(signers[10]).burn(signers[1].address, 5)).reverted;
    expect(await token.connect(owner).balanceOf(signers[1].address)).equal(9);
  });
});