import { expect } from 'chai'
import BN from 'bn.js'
import { Address, Hash } from '.'
import { Channel } from './channelEntry'

const EMPTY_ADDRESS = new Address(new Uint8Array({ length: Address.SIZE }))
const partyA = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
const partyB = Address.fromString('0xbbCFC0fA0EBaa540e741dCA297368B2000089E2E')

describe('Channel', function () {
  it('should be empty', function () {
    const channelEntry = Channel.deserialize(new Uint8Array({ length: Channel.SIZE }))

    expect(channelEntry.self.toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(channelEntry.counterparty.toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(channelEntry.deposit.toString()).to.equal('0')
    expect(channelEntry.partyABalance.toString()).to.equal('0')
    expect(channelEntry.closureTime.toString()).to.equal('0')
    expect(channelEntry.stateCounter.toString()).to.equal('0')
    expect(channelEntry.closureByPartyA).to.be.false
    expect(channelEntry.openedAt.toString()).to.equal('0')
    expect(channelEntry.closedAt.toString()).to.equal('0')
  })

  it('should contain the right values', function () {
    const channelEntry = Channel.deserialize(
      Channel.fromObject({
        partyA,
        partyB,
        deposit: new BN(10),
        partyABalance: new BN(3),
        closureTime: new BN(10),
        stateCounter: new BN(50),
        closureByPartyA: true,
        openedAt: new BN(1),
        closedAt: new BN(2),
        commitment: Hash.create(new Uint8Array())
      }).serialize()
    )

    expect(channelEntry.self.toHex()).to.equal(partyA.toHex())
    expect(channelEntry.counterparty.toHex()).to.equal(partyB.toHex())
    expect(channelEntry.deposit.toString()).to.equal('10')
    expect(channelEntry.partyABalance.toString()).to.equal('3')
    expect(channelEntry.closureTime.toString()).to.equal('10')
    expect(channelEntry.stateCounter.toString()).to.equal('50')
    expect(channelEntry.closureByPartyA).to.be.true
    expect(channelEntry.openedAt.toString()).to.equal('1')
    expect(channelEntry.closedAt.toString()).to.equal('2')
  })
})
