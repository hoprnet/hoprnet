import { stringToU8a } from '../u8a'
import { Address, ChannelEntry, PublicKey, Hash } from '.'
import { BigNumber } from 'bignumber.js'
import assert from 'assert'

const EMPTY_ADDRESS = new Address(new Uint8Array({ length: Address.SIZE }))

export const PARTY_A = PublicKey.fromPrivKey(
  stringToU8a('0x0f1b0de97ef1e907d8152bdfdaa39b4bb5879d5d48d152a84421bd2f9ccb3877')
)

export const PARTY_B = PublicKey.fromPrivKey(
  stringToU8a('0x4c6a00ceb8e3c0c4c528839f88f2eff948dd8df37e067a8b6f222c6496bdb7b0')
)

const CHANNEL_AB = {
  args: {
    partyA: PARTY_A.toAddress().toHex(),
    partyB: PARTY_B.toAddress().toHex(),
    newState: {
      partyABalance: new BigNumber('3'),
      partyBBalance: new BigNumber('0'),
      partyACommitment: Hash.create(new TextEncoder().encode('commA')).toHex(),
      partyBCommitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
      partyATicketEpoch: new BigNumber('1'),
      partyBTicketEpoch: new BigNumber('0'),
      partyATicketIndex: new BigNumber('0'),
      partyBTicketIndex: new BigNumber('0'),
      status: 1,
      channelEpoch: new BigNumber('0'),
      closureTime: new BigNumber('0'),
      closureByPartyA: false
    }
  }
}

const CHANNEL_AB_INVERTED = {
  args: {
    partyA: PARTY_B.toAddress().toHex(),
    partyB: PARTY_A.toAddress().toHex(),
    newState: {
      partyABalance: new BigNumber('3'),
      partyBBalance: new BigNumber('0'),
      partyACommitment: Hash.create(new TextEncoder().encode('commA')).toHex(),
      partyBCommitment: new Hash(new Uint8Array({ length: Hash.SIZE })).toHex(),
      partyATicketEpoch: new BigNumber('1'),
      partyBTicketEpoch: new BigNumber('0'),
      partyATicketIndex: new BigNumber('0'),
      partyBTicketIndex: new BigNumber('0'),
      status: 1,
      channelEpoch: new BigNumber('0'),
      closureTime: new BigNumber('0'),
      closureByPartyA: false
    }
  }
}

describe('ChannelEntry', function () {
  it('should be empty', function () {
    const channelEntry = ChannelEntry.deserialize(new Uint8Array({ length: ChannelEntry.SIZE }))

    assert(channelEntry.partyA.eq(EMPTY_ADDRESS))
    assert(channelEntry.partyB.eq(EMPTY_ADDRESS))
    assert(channelEntry.partyBBalance.toBN().eqn(0))
    assert(channelEntry.partyABalance.toBN().eqn(0))
    assert(channelEntry.closureTime.toBN().eqn(0))
    assert(channelEntry.closureByPartyA == false)
  })

  it('should produce correct channel entries from SC events', function () {
    const firstChannelEntry = ChannelEntry.fromSCEvent(CHANNEL_AB)
    const secondChannelEntry = ChannelEntry.fromSCEvent(CHANNEL_AB_INVERTED)

    assert(firstChannelEntry.partyA.eq(secondChannelEntry.partyA))
    assert(firstChannelEntry.partyB.eq(secondChannelEntry.partyB))
  })
})
