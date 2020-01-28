// import { time } from "@openzeppelin/test-helpers";
// using @openzeppelin/test-helpers "time" causes tests to fail
const HoprChannels = artifacts.require("HoprChannels");
const HoprToken_3 = artifacts.require("HoprToken");

module.exports = (async deployer => {
  const token = await HoprToken_3.deployed();
  const secsClosure = 172800; // 2 days

  await deployer.deploy(HoprChannels, token.address, secsClosure);
}) as Truffle.Migration;
