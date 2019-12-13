const HoprChannel = artifacts.require("HoprChannel");

module.exports = (deployer => {
  deployer.deploy(HoprChannel);
}) as Truffle.Migration;
