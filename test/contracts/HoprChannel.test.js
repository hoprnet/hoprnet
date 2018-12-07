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
contract('HoprChannel contract', ([sender, node1, node2, node3, node4, node6, node7, node8, receiver]) => {

    let hoprChannelInstance;

    before(async () => {
        hoprChannelInstance = await HoprChannel.new();
    });

    describe('verify initial deployment state', () => {

    });

    describe('stake ether for PC funds', () => {
        let tx = [];

        it('stake ether in HoprChannel contract', async () => {
            tx[0] = await hoprChannelInstance.stakeEther({from: sender, value: web3.utils.toWei("2", "ether")});
            tx[1] = await hoprChannelInstance.stakeEther({from: node1, value: web3.utils.toWei("2", "ether")});
            tx[2] = await hoprChannelInstance.stakeEther({from: node2, value: web3.utils.toWei("2", "ether")});
            tx[3] = await hoprChannelInstance.stakeEther({from: receiver, value: web3.utils.toWei("2", "ether")});
        });

        it('verify balance of HoprChannel contract', async () => {
            const balance = await web3.eth.getBalance(hoprChannelInstance.address);

            assert.equal(balance.toString(), Number(8e18).toString(), 'balance !=');
        });

        it('verify state of each account', async () => {
            let result = await hoprChannelInstance.states(sender);

            assert.isTrue(result.isSet, 'isSet should be true');
            assert.equal(result.openChannels, 0, 'openChannels !=');
            assert.equal(result.stakedEther.toString(), Number(2e18).toString(), 'stakedEther !=');
            assert.equal(result.counter, 0, 'counter !=');

            result = await hoprChannelInstance.states(node1);

            assert.isTrue(result.isSet, 'isSet should be true');
            assert.equal(result.openChannels, 0, 'openChannels !=');
            assert.equal(result.stakedEther.toString(), Number(2e18).toString(), 'stakedEther !=');
            assert.equal(result.counter, 0, 'counter !=');

            result = await hoprChannelInstance.states(node2);

            assert.isTrue(result.isSet, 'isSet should be true');
            assert.equal(result.openChannels, 0, 'openChannels !=');
            assert.equal(result.stakedEther.toString(), Number(2e18).toString(), 'stakedEther !=');
            assert.equal(result.counter, 0, 'counter !=');

            result = await hoprChannelInstance.states(receiver);

            assert.isTrue(result.isSet, 'isSet should be true');
            assert.equal(result.openChannels, 0, 'openChannels !=');
            assert.equal(result.stakedEther.toString(), Number(2e18).toString(), 'stakedEther !=');
            assert.equal(result.counter, 0, 'counter !=');

            result = await hoprChannelInstance.states(node3);

            assert.isFalse(result.isSet, 'isSet should be false');
            assert.equal(result.openChannels, 0, 'openChannels !=');
            assert.equal(result.stakedEther.toString(), Number(0).toString(), 'stakedEther !=');
            assert.equal(result.counter, 0, 'counter !=');
        });
    });
});
