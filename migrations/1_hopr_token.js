const HoprToken = artifacts.require("HoprToken");

module.exports = async deployer => {
  await deployer.deploy(HoprToken);
};
