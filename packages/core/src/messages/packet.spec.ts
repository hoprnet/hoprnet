import { HoprDB, UINT256, u8aEquals, Balance, u8aToHex } from '@hoprnet/hopr-utils'
import { PRICE_PER_PACKET } from '@hoprnet/hopr-utils'
import { Packet, PacketHelper, PacketState, privateKeyFromPeer } from './packet.js'
import { createSecp256k1PeerId } from '@libp2p/peer-id-factory'
import assert from 'assert'
import BN from 'bn.js'

const INTERMEDIATE_HOPS: number = 3;
function createMockTickets() {
  const tags = new Set<string>()
  const db = {
    getChannelTo: () => ({
      getId: () => ({ toHex: () => '0xdeadbeef' }),
      ticketEpoch: new UINT256(new BN(0)),
      channelEpoch: new UINT256(new BN(0)),
      balance: new Balance(new BN(100).mul(PRICE_PER_PACKET))
    }),
    getCurrentTicketIndex: () => {},
    setCurrentTicketIndex: () => {},
    checkAndSetPacketTag: async (tag: Uint8Array) => {
      const tagString = u8aToHex(tag)
      if (tags.has(tagString)) {
        return true
      }

      tags.add(tagString)

      return false
    },
    storeUnacknowledgedTicket: () => Promise.resolve(),
    markPending: () => Promise.resolve(),
    getPendingBalanceTo: async () => new Balance(new BN(0)),
    storePendingAcknowledgement: () => Promise.resolve()
  }
  return { db: db as any as HoprDB }
}

describe('packet creation and transformation', function () {
  it('create packet and transform it', async function () {
    const AMOUNT = INTERMEDIATE_HOPS + 1
    const [self, ...path] = await Promise.all(Array.from({ length: AMOUNT }).map(createSecp256k1PeerId))
    const { db } = createMockTickets()
    const testMsg = new TextEncoder().encode('test')
    let packet = await PacketHelper.create(testMsg, path, self, db)
    assert(packet.ack_challenge() != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      let node_private = privateKeyFromPeer(node)
      packet = Packet.deserialize(packet.serialize(), node_private, (index == 0 ? self : path[index - 1]).toString())
      const { db } = createMockTickets()
      await PacketHelper.checkPacketTag(packet, db)

      if (packet.state() == PacketState.Final) {
        assert(index == path.length - 1)
        assert(u8aEquals(packet.plaintext(), testMsg))
      } else {
        await PacketHelper.storeUnacknowledgedTicket(packet, db)
        await PacketHelper.forwardTransform(packet, node, db)
      }
    }
  })

  it('create packet and transform it - reduced path', async function () {
    const AMOUNT = INTERMEDIATE_HOPS
    const [self, ...path] = await Promise.all(Array.from({ length: AMOUNT }).map(createSecp256k1PeerId))
    const { db } = createMockTickets()
    const testMsg = new TextEncoder().encode('test')
    let packet = await PacketHelper.create(testMsg, path, self, db)
    assert(packet.ack_challenge() != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      let node_private = privateKeyFromPeer(node)
      packet = Packet.deserialize(packet.serialize(), node_private, (index == 0 ? self : path[index - 1]).toString())
      const { db } = createMockTickets()
      await PacketHelper.checkPacketTag(packet, db)

      // Checks that packet tag is set and cannot set twice
      await assert.rejects(PacketHelper.checkPacketTag(packet, db))

      if (packet.state() == PacketState.Final) {
        assert(index == path.length - 1)
        assert(u8aEquals(packet.plaintext(), testMsg))
      } else {
        await PacketHelper.storeUnacknowledgedTicket(packet, db)
        await PacketHelper.forwardTransform(packet, node, db)
      }
    }
  })

  it('create packet and transform it - zero hop', async function () {
    const AMOUNT = 2
    const [self, ...path] = await Promise.all(Array.from({ length: AMOUNT }).map(createSecp256k1PeerId))

    const { db } = createMockTickets()
    const testMsg = new TextEncoder().encode('test')
    let packet = await PacketHelper.create(testMsg, path, self, db)
    assert(packet.ack_challenge() != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      let node_private = privateKeyFromPeer(node)
      packet = Packet.deserialize(packet.serialize(), node_private, (index == 0 ? self : path[index - 1]).toString())
      const { db } = createMockTickets()
      await PacketHelper.checkPacketTag(packet, db)

      // Checks that packet tag is set and cannot set twice
      await assert.rejects(PacketHelper.checkPacketTag(packet, db))

      if (packet.state() == PacketState.Final) {
        assert(index == path.length - 1)
        assert(u8aEquals(packet.plaintext(), testMsg))
      } else {
        await PacketHelper.storeUnacknowledgedTicket(packet, db)
        await PacketHelper.forwardTransform(packet, node, db)
      }
    }
  })

  it('create packet and transform it - false positives', async function () {
    const AMOUNT = INTERMEDIATE_HOPS + 1
    const [self, ...path] = await Promise.all(Array.from({ length: AMOUNT }).map(createSecp256k1PeerId))
    const { db } = createMockTickets()
    const testMsg = new TextEncoder().encode('test')
    const packet = await PacketHelper.create(testMsg, path, self, db)
    let path0_priv = privateKeyFromPeer(path[0])
    const transformedPacket = Packet.deserialize(packet.serialize(), path0_priv, self.toString())
    await PacketHelper.forwardTransform(transformedPacket, path[0], db)
    assert.throws(() => Packet.deserialize(transformedPacket.serialize(), path0_priv, self.toString()))
  })
})
