import { stringToU8a } from '../u8a'
import { ChannelEntry, PublicKey, Address } from '.'
import assert from 'assert'

const EMPTY_ADDRESS = new Address(new Uint8Array({ length: Address.SIZE }))

export const PARTY_A = PublicKey.fromPrivKey(
  stringToU8a('0x0f1b0de97ef1e907d8152bdfdaa39b4bb5879d5d48d152a84421bd2f9ccb3877')
)

export const PARTY_B = PublicKey.fromPrivKey(
  stringToU8a('0x4c6a00ceb8e3c0c4c528839f88f2eff948dd8df37e067a8b6f222c6496bdb7b0')
)

describe('ChannelEntry', function () {
  it('should be empty', function () {
    const channelEntry = ChannelEntry.deserialize(new Uint8Array({ length: ChannelEntry.SIZE }))

    assert(channelEntry.source.eq(EMPTY_ADDRESS))
    assert(channelEntry.destination.eq(EMPTY_ADDRESS))
    assert(channelEntry.balance.toBN().eqn(0))
    assert(channelEntry.closureTime.toBN().eqn(0))
  })
})
