const PaymentChannel = artifacts.require("PaymentChannel");

module.exports = (deployer => {
  deployer.deploy(PaymentChannel);
}) as Truffle.Migration;
