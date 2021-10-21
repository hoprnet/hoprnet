import { Packet, INTERMEDIATE_HOPS } from './packet'
import {
  HoprDB,
  u8aEquals
} from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import assert from 'assert'

function createMockTickets(_privKey: Uint8Array) {
  /*
  const acknowledge = () => {}

  
  const getChannel = (_self: PublicKey, counterparty: PublicKey) => ({
    acknowledge,
    createTicket: async (pathLength: number, challenge: Challenge) => {
      return Promise.resolve(
        Ticket.create(
          counterparty.toAddress(),
          challenge,
          new UINT256(new BN(0)),
          new UINT256(new BN(0)),
          new Balance(PRICE_PER_PACKET.muln(pathLength)),
          UINT256.fromInverseProbability(new BN(1)),
          new UINT256(new BN(0)),
          privKey
        )
      )
    },
    createDummyTicket: (challenge: Challenge) =>
      Ticket.create(
        counterparty.toAddress(),
        challenge,
        new UINT256(new BN(0)),
        new UINT256(new BN(0)),
        new Balance(new BN(0)),
        UINT256.DUMMY_INVERSE_PROBABILITY,
        new UINT256(new BN(0)),
        privKey
      )
  })
  */

  const chain = {}
  const db = HoprDB.createMock()
  return { chain, db }
}

describe('packet creation and transformation', function () {
  it('create packet and transform it', async function () {
    const AMOUNT = INTERMEDIATE_HOPS + 1
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const { db } = createMockTickets(self.privKey.marshal())

    const testMsg = new TextEncoder().encode('test')
    let packet = await Packet.create(testMsg, path, self, db)
    assert(packet.ackChallenge != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])

      const { db } = createMockTickets(self.privKey.marshal())
      await packet.checkPacketTag(db)
      assert.rejects(packet.checkPacketTag(db))

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

    const { db } = createMockTickets(self.privKey.marshal())

    const testMsg = new TextEncoder().encode('test')

    let packet = await Packet.create(testMsg, path, self, db)

    assert(packet.ackChallenge != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])

      const { db } = createMockTickets(self.privKey.marshal())

      await packet.checkPacketTag(db)

      assert.rejects(packet.checkPacketTag(db))


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

    const { db }= createMockTickets(self.privKey.marshal())

    const testMsg = new TextEncoder().encode('test')

    let packet = await Packet.create(testMsg, path, self, db)

    assert(packet.ackChallenge != null, `ack challenge must be set to track if message was sent`)

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])

      const { db } = createMockTickets(self.privKey.marshal())
      await packet.checkPacketTag(db)
      assert.rejects(packet.checkPacketTag(db))

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

    const { db } = createMockTickets(self.privKey.marshal())

    const testMsg = new TextEncoder().encode('test')

    const packet = await Packet.create(testMsg, path, self, db)

    const transformedPacket = Packet.deserialize(packet.serialize(), path[0], self)

    await transformedPacket.forwardTransform(path[0],  db)

    assert.throws(() => Packet.deserialize(transformedPacket.serialize(), path[0], self))
  })
})
