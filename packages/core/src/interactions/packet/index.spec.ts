import PeerId from 'peer-id'
import { randomBytes } from 'crypto'
import { EventEmitter } from 'events'
import BN from 'bn.js'

import { subscribeToAcknowledgements, sendAcknowledgement } from './acknowledgement'
import { PublicKey, u8aEquals, Ticket, UINT256, HoprDB, Challenge, deriveAckKeyShare } from '@hoprnet/hopr-utils'
import { Balance, createPoRValuesForSender } from '@hoprnet/hopr-utils'

import { AcknowledgementChallenge, Packet } from '../../messages'
import { PacketForwardInteraction } from './forward'
import Defer from 'p-defer'

const SECRET_LENGTH = 32

function createFakeChain(privKey: PeerId) {
  const acknowledge = () => {}

  const getChannel = (_self: PublicKey, counterparty: PublicKey) => ({
    acknowledge,
    createTicket: (amount: Balance, challenge: Challenge, _winProb: number) => {
      return Ticket.create(
        counterparty.toAddress(),
        challenge,
        new UINT256(new BN(0)),
        new UINT256(new BN(0)),
        amount,
        UINT256.fromProbability(1),
        new UINT256(new BN(0)),
        privKey.privKey.marshal()
      )
    }
  })

  return { getChannel }
}

function createFakeSendReceive(events: EventEmitter, self: PeerId) {
  const send = (destination: PeerId, protocol: any, msg: Uint8Array) => {
    events.emit('msg', msg, self, destination, protocol)
  }

  const subscribe = (protocol: string, foo: (msg: Uint8Array, sender: PeerId) => any) => {
    events.on('msg', (msg, sender, destination, protocolSubscription) => {
      if (self.equals(destination) && protocol === protocolSubscription) {
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
  const db = HoprDB.createMock()

  let events = new EventEmitter()

  afterEach(function () {
    events.removeAllListeners()
  })

  it('acknowledgement workflow', async function () {
    const [self, counterparty] = await Promise.all(
      Array.from({ length: 2 }, (_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const chainSelf = createFakeChain(self)
    const libp2pSelf = createFakeSendReceive(events, self)
    const libp2pCounterparty = createFakeSendReceive(events, counterparty)

    const secrets = Array.from({ length: 2 }, (_) => randomBytes(SECRET_LENGTH))

    const { ackChallenge, ownKey } = createPoRValuesForSender(secrets[0], secrets[1])

    const challenge = AcknowledgementChallenge.create(ackChallenge, self)

    const fakePacket = new Packet(new Uint8Array(), challenge, { serialize: () => new Uint8Array(Ticket.SIZE) } as any)

    fakePacket.ownKey = ownKey
    fakePacket.ackKey = deriveAckKeyShare(secrets[0])
    fakePacket.nextHop = counterparty.pubKey.marshal()
    fakePacket.ackChallenge = ackChallenge

    fakePacket.storeUnacknowledgedTicket(db)

    const defer = Defer()

    subscribeToAcknowledgements(libp2pSelf.subscribe, db, chainSelf as any, self, () => {
      defer.resolve()
    })

    sendAcknowledgement(fakePacket, self, libp2pCounterparty.send, counterparty)

    await defer.promise
  })

  it('packet-acknowledgement workflow', async function () {
    const [sender, relay0, relay1, relay2, receiver] = await Promise.all(
      Array.from({ length: 5 }, (_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const chainSender = createFakeChain(sender)
    const chainRelay0 = createFakeChain(relay0)
    const chainRelay1 = createFakeChain(relay1)
    const chainRelay2 = createFakeChain(relay2)
    const chainReceiver = createFakeChain(receiver)

    const libp2pSender = createFakeSendReceive(events, sender)
    const libp2pRelay0 = createFakeSendReceive(events, relay0)
    const libp2pRelay1 = createFakeSendReceive(events, relay1)
    const libp2pRelay2 = createFakeSendReceive(events, relay2)
    const libp2pReceiver = createFakeSendReceive(events, receiver)

    const testMsg = new TextEncoder().encode('testMsg')
    const packet = await Packet.create(testMsg, [relay0, relay1, relay2, receiver], sender, chainSender as any, {
      value: new Balance(new BN(0)),
      winProb: 1
    })

    const msgDefer = Defer()

    const senderInteraction = new PacketForwardInteraction(
      libp2pSender.subscribe,
      libp2pSender.send,
      sender,
      chainSender as any,
      console.log,
      db
    )

    // @ts-expect-error
    const relay0Interaction = new PacketForwardInteraction(
      libp2pRelay0.subscribe,
      libp2pRelay0.send,
      relay0,
      chainRelay0 as any,
      console.log,
      db
    )

    // @ts-expect-error
    const relay1Interaction = new PacketForwardInteraction(
      libp2pRelay1.subscribe,
      libp2pRelay1.send,
      relay1,
      chainRelay1 as any,
      console.log,
      db
    )

    // @ts-expect-error
    const relay2Interaction = new PacketForwardInteraction(
      libp2pRelay2.subscribe,
      libp2pRelay2.send,
      relay2,
      chainRelay2 as any,
      console.log,
      db
    )

    // @ts-expect-error
    const receiverInteraction = new PacketForwardInteraction(
      libp2pReceiver.subscribe,
      libp2pReceiver.send,
      receiver,
      chainReceiver as any,
      (msg: Uint8Array) => {
        if (u8aEquals(msg, testMsg)) {
          msgDefer.resolve()
        }
      },
      db
    )

    senderInteraction.interact(relay0, packet)

    await msgDefer.promise
  })
})
