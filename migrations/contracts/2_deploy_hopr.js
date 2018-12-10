/* eslint-disable prefer-destructuring */
/**
 * Migration
 */
const cnf = require('../../config/contract-hopr.json');
const HoprChannel = artifacts.require('./HoprChannel.sol');

module.exports = function (deployer, network, accounts) { // eslint-disable-line
    deployer.deploy(HoprChannel).then(() => {
        // do something
    });
};
