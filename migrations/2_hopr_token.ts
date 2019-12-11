const HoprToken = artifacts.require("HoprToken");

module.exports = (deployer => {
  deployer.deploy(HoprToken);
}) as Truffle.Migration;

// because of https://stackoverflow.com/questions/40900791/cannot-redeclare-block-scoped-variable-in-unrelated-files
export {};
