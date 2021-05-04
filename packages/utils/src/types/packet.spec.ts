import { Packet, MAX_HOPS } from './packet'
import { HoprDB } from '..'
import PeerId from 'peer-id'
import { Ticket, UINT256, Balance, PublicKey, Address } from '.'
import BN from 'bn.js'
import { u8aEquals } from '../u8a'
import assert from 'assert'

function createMockTickets(privKey: Uint8Array) {
  const acknowledge = () => {}

  const getChannel = (_self: PublicKey, counterparty: PublicKey) => ({
    acknowledge,
    createTicket: (amount: Balance, challenge: Address, _winProb: number) => {
      return Ticket.create(
        counterparty.toAddress(),
        challenge,
        new UINT256(new BN(0)),
        new UINT256(new BN(0)),
        amount,
        Ticket.fromProbability(1),
        new UINT256(new BN(0)),
        privKey
      )
    }
  })

  return { getChannel }
}

describe('packet creation and transformation', function () {
  it('create packet and transform it', async function () {
    const AMOUNT = MAX_HOPS + 1
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const chain = createMockTickets(self.privKey.marshal())

    const testMsg = new TextEncoder().encode('test')

    let packet = await Packet.create(testMsg, path, self, chain as any, {
      value: new Balance(new BN(0)),
      winProb: 1
    })

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])

      const db = HoprDB.createMock()

      await packet.checkPacketTag(db)

      assert.rejects(packet.checkPacketTag(db))

      const chain = createMockTickets(node.privKey.marshal())

      if (packet.isReceiver) {
        assert(index == path.length - 1)

        assert(u8aEquals(packet.plaintext, testMsg))
      } else {
        await packet.storeUnacknowledgedTicket(db)

        await packet.forwardTransform(node, chain as any)
      }
    }
  })

  it('create packet and transform it - reduced path', async function () {
    const AMOUNT = MAX_HOPS
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const chain = createMockTickets(self.privKey.marshal())

    const testMsg = new TextEncoder().encode('test')

    let packet = await Packet.create(testMsg, path, self, chain as any, {
      value: new Balance(new BN(0)),
      winProb: 1
    })

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])

      const db = HoprDB.createMock()

      await packet.checkPacketTag(db)

      assert.rejects(packet.checkPacketTag(db))

      const chain = createMockTickets(node.privKey.marshal())

      if (packet.isReceiver) {
        assert(index == path.length - 1)

        assert(u8aEquals(packet.plaintext, testMsg))
      } else {
        await packet.storeUnacknowledgedTicket(db)

        await packet.forwardTransform(node, chain as any)
      }
    }
  })

  it('create packet and transform it - zero hop', async function () {
    const AMOUNT = 2
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const chain = createMockTickets(self.privKey.marshal())

    const testMsg = new TextEncoder().encode('test')

    let packet = await Packet.create(testMsg, path, self, chain as any, {
      value: new Balance(new BN(0)),
      winProb: 1
    })

    for (const [index, node] of path.entries()) {
      packet = Packet.deserialize(packet.serialize(), node, index == 0 ? self : path[index - 1])

      const db = HoprDB.createMock()

      await packet.checkPacketTag(db)

      assert.rejects(packet.checkPacketTag(db))

      const chain = createMockTickets(node.privKey.marshal())

      if (packet.isReceiver) {
        assert(index == path.length - 1)

        assert(u8aEquals(packet.plaintext, testMsg))
      } else {
        await packet.storeUnacknowledgedTicket(db)

        await packet.forwardTransform(node, chain as any)
      }
    }
  })

  it('create packet and transform it - false positives', async function () {
    const AMOUNT = MAX_HOPS + 1
    const [self, ...path] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const chain = createMockTickets(self.privKey.marshal())

    const testMsg = new TextEncoder().encode('test')

    const packet = await Packet.create(testMsg, path, self, chain as any, {
      value: new Balance(new BN(0)),
      winProb: 1
    })

    const transformedPacket = Packet.deserialize(packet.serialize(), path[0], self)

    await transformedPacket.forwardTransform(path[0], chain as any)

    assert.throws(() => Packet.deserialize(transformedPacket.serialize(), path[0], self))
  })
})
