import { expect } from 'chai'
import BN from 'bn.js'
import ChannelEntry from './channelEntry'

describe('ChannelEntry', function () {
  it('should be empty', function () {
    const channelEntry = new ChannelEntry(undefined, {
      blockNumber: new BN(0),
      transactionIndex: new BN(0),
      logIndex: new BN(0),
      deposit: new BN(0),
      partyABalance: new BN(0),
      closureTime: new BN(0),
      stateCounter: new BN(0),
      closureByPartyA: false
    })

    expect(channelEntry.blockNumber.toString()).to.equal('0')
    expect(channelEntry.transactionIndex.toString()).to.equal('0')
    expect(channelEntry.logIndex.toString()).to.equal('0')
    expect(channelEntry.deposit.toString()).to.equal('0')
    expect(channelEntry.partyABalance.toString()).to.equal('0')
    expect(channelEntry.closureTime.toString()).to.equal('0')
    expect(channelEntry.stateCounter.toString()).to.equal('0')
    expect(channelEntry.closureByPartyA).to.be.false
  })

  it('should contain the right values', function () {
    const channelEntry = new ChannelEntry(undefined, {
      blockNumber: new BN(1),
      transactionIndex: new BN(2),
      logIndex: new BN(3),
      deposit: new BN(10),
      partyABalance: new BN(3),
      closureTime: new BN(10),
      stateCounter: new BN(50),
      closureByPartyA: true
    })

    expect(channelEntry.blockNumber.toString()).to.equal('1')
    expect(channelEntry.transactionIndex.toString()).to.equal('2')
    expect(channelEntry.logIndex.toString()).to.equal('3')
    expect(channelEntry.deposit.toString()).to.equal('10')
    expect(channelEntry.partyABalance.toString()).to.equal('3')
    expect(channelEntry.closureTime.toString()).to.equal('10')
    expect(channelEntry.stateCounter.toString()).to.equal('50')
    expect(channelEntry.closureByPartyA).to.be.true
  })
})
