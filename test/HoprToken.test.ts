import BN = require("bn.js");
import { HoprTokenContract } from "../types/truffle-contracts";

const HoprToken = artifacts.require("HoprToken") as HoprTokenContract;

contract("HoprToken", _accounts => {
  it("should be named 'HOPR'", async () => {
    const contract = await HoprToken.deployed();

    expect(await contract.name()).to.be.equal("HOPR");
  });

  it("should have symbol 'HOPR'", async () => {
    const contract = await HoprToken.deployed();

    expect(await contract.symbol()).to.be.equal("HOPR");
  });

  it("should have a supply of '100 million'", async () => {
    const contract = await HoprToken.deployed();
    const totalSupply = await contract.totalSupply();

    assert.isTrue(totalSupply.eq(new BN(100e6)));
  });
});
