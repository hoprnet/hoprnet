/* eslint-disable prefer-destructuring */
/**
 * Migration - ICO & Token
 * Deploys a Token contract, then the ICO contract passing the address of the previously deployed token.
 * Adds minter role to ico contract. Renounces minter role for the original token contract deployer (accounts[0])
 */
const cnf = require('../../config/contract-hopr.json');
const HoprChannel = artifacts.require('./HoprChannel.sol');

module.exports = function (deployer, network, accounts) { // eslint-disable-line
    deployer.deploy(HoprChannel).then(() => {
        // do something
    });
};
