import { type Address, ChannelEntry, PublicKey } from '.'
import assert from 'assert'
import BN from 'bn.js'

const PARTY_A = PublicKey.fromPrivKeyString('0x0f1b0de97ef1e907d8152bdfdaa39b4bb5879d5d48d152a84421bd2f9ccb3877')
const PARTY_B = PublicKey.fromPrivKeyString('0x4c6a00ceb8e3c0c4c528839f88f2eff948dd8df37e067a8b6f222c6496bdb7b0')

const COMMITMENT = '0xffab46f058090de082a086ea87c535d34525a48871c5a2024f80d0ac850f81ef'

function keyFor(addr: Address): Promise<PublicKey> {
  if (PARTY_A.toAddress().eq(addr)) {
    return Promise.resolve(PARTY_A)
  } else if (PARTY_B.toAddress().eq(addr)) {
    return Promise.resolve(PARTY_B)
  }
}

describe('ChannelEntry', function () {
  it('from SC event, serialize, deserialize', async function () {
    const entry = await ChannelEntry.fromSCEvent(
      {
        args: {
          source: PARTY_A.toAddress().toHex(),
          destination: PARTY_B.toAddress().toHex(),
          newState: {
            balance: new BN(2),
            commitment: COMMITMENT,
            ticketEpoch: new BN(3),
            ticketIndex: new BN(4),
            status: 1,
            channelEpoch: new BN(5),
            closureTime: new BN(6)
          }
        }
      },
      keyFor
    )

    const serialized = entry.serialize()

    assert(serialized.length == ChannelEntry.SIZE)

    const deserialized = ChannelEntry.deserialize(serialized)

    assert(entry.source.eq(deserialized.source))
    assert(entry.destination.eq(deserialized.destination))
    assert(entry.balance.eq(deserialized.balance))
    assert(entry.commitment.eq(deserialized.commitment))
    assert(entry.ticketEpoch.eq(deserialized.ticketEpoch))
    assert(entry.ticketIndex.eq(deserialized.ticketIndex))
    assert(entry.status.toString() === deserialized.status.toString())
    assert(entry.channelEpoch.eq(deserialized.channelEpoch))
    assert(entry.closureTime.eq(deserialized.closureTime))
  })
})
