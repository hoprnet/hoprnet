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

    expect(channelEntry.parties[0].toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(channelEntry.parties[1].toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(channelEntry.deposit.toString()).to.equal('0')
    expect(channelEntry.partyABalance.toString()).to.equal('0')
    expect(channelEntry.closureTime.toString()).to.equal('0')
    expect(channelEntry.stateCounter.toString()).to.equal('0')
    expect(channelEntry.closureByPartyA).to.be.false
    expect(channelEntry.openedAt.toString()).to.equal('0')
    expect(channelEntry.closedAt.toString()).to.equal('0')
  })

  it('should contain the right values', function () {
    const parties: [Address, Address] = [partyA, partyB]

    const channelEntry = ChannelEntry.deserialize(
      new ChannelEntry(parties, new BN(10), new BN(3), new BN(10), new BN(50), true, new BN(1), new BN(2)).serialize()
    )

    expect(channelEntry.parties[0].toHex()).to.equal(partyA.toHex())
    expect(channelEntry.parties[1].toHex()).to.equal(partyB.toHex())
    expect(channelEntry.deposit.toString()).to.equal('10')
    expect(channelEntry.partyABalance.toString()).to.equal('3')
    expect(channelEntry.closureTime.toString()).to.equal('10')
    expect(channelEntry.stateCounter.toString()).to.equal('50')
    expect(channelEntry.closureByPartyA).to.be.true
    expect(channelEntry.openedAt.toString()).to.equal('1')
    expect(channelEntry.closedAt.toString()).to.equal('2')
  })
})
