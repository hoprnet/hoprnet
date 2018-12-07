/**
 * Test HoprChannel.sol
 * @author Validity Labs AG <info@validitylabs.org>
 */

import {expectThrow, getEvents, BigNumber} from './helpers/tools';

const HoprChannel = artifacts.require('./HoprChannel');

const should = require('chai') // eslint-disable-line
    .use(require('chai-as-promised'))
    .use(require('chai-bignumber')(BigNumber))
    .should();

/**
 * HoprChannel contract
 */
contract('HoprChannel contract', ([owner, sender, node1, node2, node3, node4, receiver]) => {

    let hoprChannelInstance;

    before(() => {
        hoprChannelInstance = new HoprChannel.new();
    });

    describe('verify initial deployment state', () => {

    });
});
