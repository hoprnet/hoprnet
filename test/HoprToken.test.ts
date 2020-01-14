import {
  HoprTokenContract,
  HoprTokenInstance
} from "../types/truffle-contracts";
import { expectEvent } from "@openzeppelin/test-helpers";

const HoprToken: HoprTokenContract = artifacts.require("HoprToken");

contract("HoprToken", function([owner, minter]) {
  let hoprToken: HoprTokenInstance;

  before(async function() {
    hoprToken = await HoprToken.deployed();
  });

  it("should be named 'HOPR Token'", async function() {
    expect(await hoprToken.name()).to.be.equal("HOPR Token", "wrong name");
  });

  it("should have symbol 'HOPR'", async function() {
    expect(await hoprToken.symbol()).to.be.equal("HOPR", "wrong symbol");
  });

  it("should have a supply of '0'", async function() {
    const totalSupply = await hoprToken.totalSupply();

    expect(totalSupply.isZero()).to.be.equal(true, "wrong total supply");
  });

  it("should replace minter to 'minter'", async function() {
    const response = await hoprToken.replaceMinter(minter);

    await expectEvent(response, "MinterRemoved", {
      account: owner
    });
    await expectEvent(response, "MinterAdded", {
      account: minter
    });

    expect(await hoprToken.isMinter(minter)).to.be.equal(true, "wrong minter");
  });
});
