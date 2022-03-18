import { type Address, ChannelEntry, PublicKey } from '.'
import assert from 'assert'

const PARTY_A = PublicKey.fromString(
  '0x0443a3958ac66a3b2ab89fcf90bc948a8b8be0e0478d21574d077ddeb11f4b1e9f2ca21d90bd66cee037255480a514b91afae89e20f7f7fa7353891cc90a52bf6e'
)
const PARTY_B = PublicKey.fromString(
  '0x04f16fd6701aea01032716377d52d8213497c118f99cdd1c3c621b2795cac8681606b7221f32a8c5d2ef77aa783bec8d96c11480acccabba9e8ee324ae2dfe92bb'
)

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
            balance: '2',
            commitment: COMMITMENT,
            ticketEpoch: '3',
            ticketIndex: '4',
            status: 1,
            channelEpoch: '5',
            closureTime: '6'
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
