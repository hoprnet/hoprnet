import { expect } from 'chai'
import { Address, ChannelEntry } from '.'

const EMPTY_ADDRESS = new Address(new Uint8Array({ length: Address.SIZE }))

describe('ChannelEntry', function () {
  it('should be empty', function () {
    const channelEntry = ChannelEntry.deserialize(new Uint8Array({ length: ChannelEntry.SIZE }))

    expect(channelEntry.partyA.toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(channelEntry.partyB.toHex()).to.equal(EMPTY_ADDRESS.toHex())
    expect(channelEntry.partyBBalance.toBN().toString()).to.equal('0')
    expect(channelEntry.partyABalance.toBN().toString()).to.equal('0')
    expect(channelEntry.closureTime.toBN().toString()).to.equal('0')
    expect(channelEntry.closureByPartyA).to.be.false
  })
})
