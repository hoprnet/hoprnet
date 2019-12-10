/**
 * Migrate Token
 */
const HoprToken = artifacts.require("./HoprToken.sol");

module.exports = function(deployer: any, _network: any, _accounts: any) {
  deployer.deploy(HoprToken);
};
