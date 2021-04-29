import LevelUp from 'levelup'
import PeerId from 'peer-id'
import Memdown from 'memdown'
import { randomBytes } from 'crypto'
import { EventEmitter } from 'events'

import { subscribeToAcknowledgements, sendAcknowledgement } from './acknowledgement'
import { Balance, createFirstChallenge, PublicKey } from '@hoprnet/hopr-utils'
import { Ticket } from '@hoprnet/hopr-core-ethereum'

import { Challenge } from '../../messages/challenge'
import { Packet } from '../../messages/packet'
import { PacketForwardInteraction } from './forward'
import Defer from 'p-defer'

const SECRET_LENGTH = 32

function createFakeChain() {
  const acknowledge = () => {}
  const createTicket = () => {}

  const getChannel = () => ({
    acknowledge,
    createTicket
  })

  return { getChannel }
}

function createFakeSendReceive(events: EventEmitter, self: PeerId) {
  const send = (destination: PeerId, _protocol: any, msg: Uint8Array) => {
    events.emit('msg', msg, self, destination)
  }

  const subscribe = (_protocol: any, foo: (msg: Uint8Array, sender: PeerId) => any) => {
    events.on('msg', (msg, sender, destination) => {
      if (self.equals(destination)) {
        foo(msg, sender)
      }
    })
  }

  return {
    send,
    subscribe
  }
}

describe('packet interaction', function () {
  let self: PeerId
  let counterparty: PeerId

  const db = LevelUp(Memdown())

  let events = new EventEmitter()

  before(async function () {
    ;[self, counterparty] = await Promise.all(Array.from({ length: 2 }, (_) => PeerId.create({ keyType: 'secp256k1' })))
  })

  it('acknowledgement workflow', async function () {
    const chain = createFakeChain()
    const libp2pSelf = createFakeSendReceive(events, self)
    const libp2pCounterparty = createFakeSendReceive(events, counterparty)

    const secrets = Array.from({ length: 2 }, (_) => randomBytes(SECRET_LENGTH))

    const { ackChallenge } = createFirstChallenge(secrets)

    const challenge = Challenge.create(ackChallenge, self)

    const fakePacket = new Packet(new Uint8Array(), challenge, { serialize: () => new Uint8Array(Ticket.SIZE) } as any)

    fakePacket.ownKey = secrets[0]
    fakePacket.nextHop = counterparty.pubKey.marshal()
    fakePacket.ackChallenge = ackChallenge

    fakePacket.storeUnacknowledgedTicket(db)

    const defer = Defer()

    subscribeToAcknowledgements(libp2pSelf.subscribe, db, chain as any, self, () => {
      defer.resolve()
    })

    sendAcknowledgement(fakePacket, self, libp2pCounterparty.send, counterparty)

    await defer.promise
  })

  // it('packet-acknowledgement workflow', async function () {
  //   const [sender, relay0, relay1, relay2, receiver] = await Promise.all(
  //     Array.from({ length: 5 }, (_) => PeerId.create({ keyType: 'secp256k1' }))
  //   )

  //   const chain = createFakeChain()
  //   const libp2p = createFakeSendReceive(self, counterparty)

  //   const testMsg = new TextEncoder().encode('testMsg')
  //   const packet = await Packet.create(testMsg, [relay0, relay1, receiver], sender, chain as any, {
  //     value: new Balance(new BN(0)),
  //     winProb: 1
  //   })

  //   const forward = new PacketForwardInteraction(libp2p.subscribe, libp2p.send, sender, chain as any, console.log, db)

  //   console.log(packet)
  // })
})
