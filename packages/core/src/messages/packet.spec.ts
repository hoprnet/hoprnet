import { Packet, INTERMEDIATE_HOPS } from './packet'
import { HoprDB, UINT256, u8aEquals } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import assert from 'assert'
import BN from 'bn.js'

function createMockTickets() {
  const db = {
    getChannelTo: () => ({
      getId: () => ({ toHex: () => '0xdeadbeef' }),
      ticketEpoch: new UINT256(new BN(0)),
      channelEpoch: new UINT256(new BN(0))
    }),
    getCurrentTicketIndex: () => {},
    setCurrentTicketIndex: () => {},
    checkAndSetPacketTag: () => Promise.resolve(false),
    storeUnacknowledgedTicket: () => Promise.resolve(),
    markPending: () => Promise.resolve()
  }
  return { db: db as any as HoprDB }
}

describe('packet creation and transformation', function () {
  it('create packet and transform it', async function () {
    const AMOUNT = INTERMEDIATE_HOPS + 1
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )
    const { db } = createMockTickets()
    const testMsg = new TextEncoder().encode('test')
    let packet = await Packet.create(testMsg, path, self, db)
    assert(packet.ackChallenge != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])
      const { db } = createMockTickets()
      await packet.checkPacketTag(db)

      if (packet.isReceiver) {
        assert(index == path.length - 1)
        assert(u8aEquals(packet.plaintext, testMsg))
      } else {
        await packet.storeUnacknowledgedTicket(db)
        await packet.forwardTransform(node, db)
      }
    }
  })

  it('create packet and transform it - reduced path', async function () {
    const AMOUNT = INTERMEDIATE_HOPS
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )
    const { db } = createMockTickets()
    const testMsg = new TextEncoder().encode('test')
    let packet = await Packet.create(testMsg, path, self, db)
    assert(packet.ackChallenge != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])
      const { db } = createMockTickets()
      await packet.checkPacketTag(db)

      if (packet.isReceiver) {
        assert(index == path.length - 1)
        assert(u8aEquals(packet.plaintext, testMsg))
      } else {
        await packet.storeUnacknowledgedTicket(db)
        await packet.forwardTransform(node, db)
      }
    }
  })

  it('create packet and transform it - zero hop', async function () {
    const AMOUNT = 2
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const { db } = createMockTickets()
    const testMsg = new TextEncoder().encode('test')
    let packet = await Packet.create(testMsg, path, self, db)
    assert(packet.ackChallenge != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])
      const { db } = createMockTickets()
      await packet.checkPacketTag(db)
// Checks that packet tag is set and cannot set twice
await assert.rejects(packet.checkPacketTag(db))
      if (packet.isReceiver) {
        assert(index == path.length - 1)
        assert(u8aEquals(packet.plaintext, testMsg))
      } else {
        await packet.storeUnacknowledgedTicket(db)
        await packet.forwardTransform(node, db)
      }
    }
  })

  it('create packet and transform it - false positives', async function () {
    const AMOUNT = INTERMEDIATE_HOPS + 1
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )
    const { db } = createMockTickets()
    const testMsg = new TextEncoder().encode('test')
    const packet = await Packet.create(testMsg, path, self, db)
    const transformedPacket = Packet.deserialize(packet.serialize(), path[0], self)
    await transformedPacket.forwardTransform(path[0], db)
    assert.throws(() => Packet.deserialize(transformedPacket.serialize(), path[0], self))
  })
})
