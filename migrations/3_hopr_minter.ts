// import { time } from "@openzeppelin/test-helpers";
// using @openzeppelin/test-helpers "time" causes tests to fail
const HoprMinter = artifacts.require("HoprMinter");
const HoprToken_4 = artifacts.require("HoprToken");

module.exports = (async deployer => {
  const token = await HoprToken_4.deployed();
  const maxAmount = web3.utils.toWei("100000000", "ether");
  const duration = web3.utils
    .toBN("365")
    .mul(web3.utils.toBN("24"))
    .mul(web3.utils.toBN("60"))
    .mul(web3.utils.toBN("60"));

  await deployer.deploy(HoprMinter, token.address, maxAmount, duration);
}) as Truffle.Migration;
