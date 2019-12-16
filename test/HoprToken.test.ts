import BN = require("bn.js");
import {
  HoprTokenContract,
  HoprTokenInstance
} from "../types/truffle-contracts";

const HoprToken: HoprTokenContract = artifacts.require("HoprToken");

contract("HoprToken", _accounts => {
  let hoprToken: HoprTokenInstance;

  before(async () => {
    hoprToken = await HoprToken.deployed();
  });

  it("should be named 'HOPR Token'", async () => {
    expect(await hoprToken.name()).to.be.equal("HOPR Token");
  });

  it("should have symbol 'HOPR'", async () => {
    expect(await hoprToken.symbol()).to.be.equal("HOPR");
  });

  it("should have a supply of '100 million'", async () => {
    const totalSupply = await hoprToken
      .totalSupply()
      .then(res => res.toString());

    expect(totalSupply).to.be.equal(web3.utils.toWei("100000000", "ether"));
  });
});
