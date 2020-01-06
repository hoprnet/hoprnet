// import { time } from "@openzeppelin/test-helpers";
// using @openzeppelin/test-helpers "time" causes tests to fail
const PaymentChannel = artifacts.require("PaymentChannel");
const HoprToken_ = artifacts.require("HoprToken");

module.exports = (async deployer => {
  const token = await HoprToken_.deployed();
  const oneWeek = 604800;
  const twoDays = 172800;

  await deployer.deploy(PaymentChannel, token.address, oneWeek, twoDays);
}) as Truffle.Migration;
