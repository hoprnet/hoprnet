/* eslint-disable prefer-destructuring */
/**
 * Migration
 */
// const cnf = require('../../config/contract-hopr.json');

declare let artifacts: any;
const HoprChannel = artifacts.require("./HoprChannel.sol");

module.exports = function(deployer: any, _network: any, _accounts: any) {
  deployer.deploy(HoprChannel);
};
