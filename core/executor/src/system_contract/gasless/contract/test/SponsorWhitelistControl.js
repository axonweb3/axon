const { expect } = require("chai");

describe("SponsorWhitelistControl contract", function () {
  it("Add Remove priviledge", async function () {
    const [owner, user] = await ethers.getSigners();

    const SponsorWhitelistControl = await ethers.getContractFactory("SponsorWhitelistControl");

    const hardhatSponsorWhitelistControl = await SponsorWhitelistControl.deploy();

    const isWhitelisted = await hardhatSponsorWhitelistControl.isWhitelisted(owner.address, owner.address);
    expect(isWhitelisted).to.equal(false);

    const rtn1 = await hardhatSponsorWhitelistControl.setSponsorForGas(owner.address, 1200, {value: 2000});

    const isWhitelisted1 = await hardhatSponsorWhitelistControl.isWhitelisted(owner.address, owner.address);
    expect(isWhitelisted1).to.equal(false);

    await hardhatSponsorWhitelistControl.addPrivilege([user.address]);
    const isWhitelisted2 = await hardhatSponsorWhitelistControl.isWhitelisted(owner.address, user.address);
    expect(isWhitelisted2).to.equal(true);

    await hardhatSponsorWhitelistControl.removePrivilege([user.address]);
    const isWhitelisted3 = await hardhatSponsorWhitelistControl.isWhitelisted(owner.address, user.address);
    expect(isWhitelisted3).to.equal(false);
  });

  it("Set sponsor for gas", async function () {
    const [owner, contract, sponsor1] = await ethers.getSigners();

    const SponsorWhitelistControl = await ethers.getContractFactory("SponsorWhitelistControl");

    const hardhatSponsorWhitelistControl = await SponsorWhitelistControl.deploy();

    const rtn1 = await hardhatSponsorWhitelistControl.setSponsorForGas(contract.address, 1200, {value: 2000});

    const sponsor = await hardhatSponsorWhitelistControl.getSponsorForGas(contract.address);
    expect(sponsor).to.equal(owner.address);
  
    const balance = await hardhatSponsorWhitelistControl.getSponsoredBalanceForGas(contract.address);
    expect(balance).to.equal(2000);
  
    const upper_bound = await hardhatSponsorWhitelistControl.getSponsoredGasFeeUpperBound(contract.address);
    expect(upper_bound).to.equal(1200);

    const rtn2 = await hardhatSponsorWhitelistControl.setSponsorForGas(contract.address, 1000, {value: 2000});
  
    const balance2 = await hardhatSponsorWhitelistControl.getSponsoredBalanceForGas(contract.address);
    expect(balance2).to.equal(4000);

    // const rtn3 = await hardhatSponsorWhitelistControl.setSponsorForGas(contract.address, 1000, {sender: sponsor1.address,value: 2000});
  });
});