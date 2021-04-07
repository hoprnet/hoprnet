import { expect } from 'chai'
import BN from 'bn.js'
import { Address } from '.'
import ChannelEntry from './channelEntry'

const EMPTY_ADDRESS = new Address(new Uint8Array({ length: Address.SIZE }))
const partyA = Address.fromString('0x55CfF15a5159239002D57C591eF4ACA7f2ACAfE6')
const partyB = Address.fromString('0xbbCFC0fA0EBaa540e741dCA297368B2000089E2E')

describe('ChannelEntry', function () {
  it('should be empty', function () {
    const channelEntry = ChannelEntry.deserialize(new Uint8Array({ length: ChannelEntry.SIZE }))

    expect(channelEntry.partyA.toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(channelEntry.partyB.toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(channelEntry.deposit.toString()).to.equal('0')
    expect(channelEntry.partyABalance.toString()).to.equal('0')
    expect(channelEntry.closureTime.toString()).to.equal('0')
    expect(channelEntry.stateCounter.toString()).to.equal('0')
    expect(channelEntry.closureByPartyA).to.be.false
    expect(channelEntry.openedAt.toString()).to.equal('0')
    expect(channelEntry.closedAt.toString()).to.equal('0')
  })

  it('should contain the right values', function () {
    const channelEntry = ChannelEntry.deserialize(
      ChannelEntry.fromObject({
        partyA,
        partyB,
        deposit: new BN(10),
        partyABalance: new BN(3),
        closureTime: new BN(10),
        stateCounter: new BN(50),
        closureByPartyA: true,
        openedAt: new BN(1),
        closedAt: new BN(2)
      }).serialize()
    )

    expect(channelEntry.partyA.toHex()).to.equal(partyA.toHex())
    expect(channelEntry.partyB.toHex()).to.equal(partyB.toHex())
    expect(channelEntry.deposit.toString()).to.equal('10')
    expect(channelEntry.partyABalance.toString()).to.equal('3')
    expect(channelEntry.closureTime.toString()).to.equal('10')
    expect(channelEntry.stateCounter.toString()).to.equal('50')
    expect(channelEntry.closureByPartyA).to.be.true
    expect(channelEntry.openedAt.toString()).to.equal('1')
    expect(channelEntry.closedAt.toString()).to.equal('2')
  })
})
