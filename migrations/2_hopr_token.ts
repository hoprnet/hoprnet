const HoprToken = artifacts.require("HoprToken");

module.exports = (deployer => {
  deployer.deploy(HoprToken);
}) as Truffle.Migration;
