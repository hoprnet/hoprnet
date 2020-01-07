// import { time } from "@openzeppelin/test-helpers";
// using @openzeppelin/test-helpers "time" causes tests to fail
const PaymentChannel = artifacts.require("PaymentChannel");
const HoprToken_ = artifacts.require("HoprToken");

module.exports = (async deployer => {
  const token = await HoprToken_.deployed();
  const secsClosure = 172800; // 2 days

  await deployer.deploy(PaymentChannel, token.address, secsClosure);
}) as Truffle.Migration;
