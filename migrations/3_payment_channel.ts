const PaymentChannel = artifacts.require("PaymentChannel");
const HoprToken_ = artifacts.require("HoprToken");

module.exports = (async deployer => {
  const token = await HoprToken_.deployed();
  const oneWeek = 604800; // using @openzeppelin/test-helpers "time" causes tests to fail

  deployer.deploy(PaymentChannel, token.address, oneWeek);
}) as Truffle.Migration;
