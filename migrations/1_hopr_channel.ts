const HoprChannel = artifacts.require("HoprChannel");

module.exports = (deployer => {
  deployer.deploy(HoprChannel);
}) as Truffle.Migration;

// because of https://stackoverflow.com/questions/40900791/cannot-redeclare-block-scoped-variable-in-unrelated-files
export {};
