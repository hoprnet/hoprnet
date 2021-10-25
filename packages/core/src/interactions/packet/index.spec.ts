import type PeerId from 'peer-id'
import { randomBytes } from 'crypto'
import { EventEmitter } from 'events'
import BN from 'bn.js'

import { subscribeToAcknowledgements, sendAcknowledgement } from './acknowledgement'
import {
  Address,
  Balance,
  Challenge,
  defer,
  HoprDB,
  PublicKey,
  Ticket,
  UINT256,
  createPoRValuesForSender,
  deriveAckKeyShare,
  UnacknowledgedTicket,
  u8aEquals,
  privKeyToPeerId,
  stringToU8a,
  ChannelEntry,
  Hash,
  ChannelStatus
} from '@hoprnet/hopr-utils'

import { AcknowledgementChallenge, Packet } from '../../messages'
import { PacketForwardInteraction } from './forward'

const SECRET_LENGTH = 32

const TEST_MESSAGE = new TextEncoder().encode('test message')

const DEFAULT_FUNDING = new Balance(new BN(1234))
const DEFAULT_TICKET_EPOCH = new UINT256(new BN(1))
const DEFAULT_INDEX = new UINT256(new BN(1))
const DEFAULT_CHANNEL_EPOCH = new UINT256(new BN(1))
const DEFAULT_CLOSURE_TIME = new UINT256(new BN(0))

const SELF = privKeyToPeerId(stringToU8a('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775'))
const RELAY0 = privKeyToPeerId(stringToU8a('0x5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca'))
const RELAY1 = privKeyToPeerId(stringToU8a('0x3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa'))
const RELAY2 = privKeyToPeerId(stringToU8a('0xdb7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92'))
const COUNTERPARTY = privKeyToPeerId(stringToU8a('0x0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc'))

const nodes: PeerId[] = [SELF, RELAY0, RELAY1, RELAY2, COUNTERPARTY]

function createFakeTicket(privKey: PeerId, challenge: Challenge, counterparty: Address, amount: Balance) {
  return Ticket.create(
    counterparty,
    challenge,
    new UINT256(new BN(0)),
    new UINT256(new BN(0)),
    amount,
    UINT256.fromInverseProbability(new BN(1)),
    new UINT256(new BN(0)),
    privKey.privKey.marshal()
  )
}

function createFakeSendReceive(events: EventEmitter, self: PeerId) {
  const send = (destination: PeerId, protocol: any, msg: Uint8Array) => {
    events.emit('msg', msg, self, destination, protocol)
  }

  const subscribe = (protocol: string, onPacket: (msg: Uint8Array, sender: PeerId) => any) => {
    events.on('msg', (msg: Uint8Array, sender: PeerId, destination: PeerId, protocolSubscription: string) => {
      if (self.equals(destination) && protocol === protocolSubscription) {
        onPacket(msg, sender)
      }
    })
  }

  return {
    send,
    subscribe
  }
}

describe('packet interaction', function () {
  const events = new EventEmitter()
  let dbs: HoprDB[] = []

  afterEach(async function () {
    events.removeAllListeners()

    await Promise.all(dbs.map((db: HoprDB) => db.close))
  })

  beforeEach(function () {
    for (const [index, node] of nodes.entries()) {
      dbs[index] = HoprDB.createMock(PublicKey.fromPeerId(node))
    }
  })

  // it('acknowledgement workflow', async function () {
  //   const secrets = Array.from({ length: 2 }, () => randomBytes(SECRET_LENGTH))
  //   const { ackChallenge, ownKey, ticketChallenge } = createPoRValuesForSender(secrets[0], secrets[1])
  //   const ticket = createFakeTicket(
  //     SELF,
  //     ticketChallenge,
  //     PublicKey.fromPeerId(COUNTERPARTY).toAddress(),
  //     new Balance(new BN(1))
  //   )
  //   const challenge = AcknowledgementChallenge.create(ackChallenge, SELF)
  //   const unack = new UnacknowledgedTicket(ticket, halfKey, SELF)
  //   await db.storeUnacknowledgedTicket(ackChallenge, unack)

  //   const libp2pSelf = createFakeSendReceive(events, SELF)
  //   const libp2pCounterparty = createFakeSendReceive(events, COUNTERPARTY)

  //   const fakePacket = new Packet(TEST_MESSAGE, challenge, ticket)

  //   fakePacket.ownKey = ownKey
  //   fakePacket.ackKey = deriveAckKeyShare(secrets[0])
  //   fakePacket.nextHop = COUNTERPARTY.pubKey.marshal()
  //   fakePacket.ackChallenge = ackChallenge
  //   fakePacket.previousHop = PublicKey.fromPeerId(SELF)

  //   fakePacket.storeUnacknowledgedTicket(db)

  //   const ackReceived = defer<void>()
  //   const ev = new EventEmitter()

  //   subscribeToAcknowledgements(libp2pSelf.subscribe, db, ev, SELF, () => {
  //     ackReceived.resolve()
  //   })

  //   sendAcknowledgement(fakePacket, SELF, libp2pCounterparty.send, COUNTERPARTY)

  //   await ackReceived.promise
  // })

  it('packet-acknowledgement workflow', async function () {
    const msgDefer = defer<void>()
    const nodes: PeerId[] = [SELF, RELAY0, RELAY1, RELAY2, COUNTERPARTY]

    let senderInteraction: PacketForwardInteraction

    for (const [index, pId] of nodes.entries()) {
      const { subscribe, send } = createFakeSendReceive(events, pId)

      if (!pId.equals(RELAY2) && !pId.equals(COUNTERPARTY)) {
        const channel = new ChannelEntry(
          PublicKey.fromPeerId(pId),
          PublicKey.fromPeerId(nodes[index + 1]),
          DEFAULT_FUNDING,
          new Hash(Uint8Array.from(randomBytes(32))),
          DEFAULT_TICKET_EPOCH,
          DEFAULT_INDEX,
          ChannelStatus.Open,
          DEFAULT_CHANNEL_EPOCH,
          DEFAULT_CLOSURE_TIME
        )

        await dbs[index].updateChannel(channel.getId(), channel)
      }

      let receive: (msg: Uint8Array) => void

      if (pId.equals(COUNTERPARTY)) {
        receive = (msg: Uint8Array) => {
          if (u8aEquals(msg, TEST_MESSAGE)) {
            msgDefer.resolve()
          }
        }
      } else {
        receive = console.log
      }

      const interaction = new PacketForwardInteraction(subscribe, send, pId, receive, dbs[index])

      if (pId.equals(SELF)) {
        senderInteraction = interaction
      }
    }

    const packet = await Packet.create(TEST_MESSAGE, [RELAY0, RELAY1, RELAY2, COUNTERPARTY], SELF, dbs[0])

    // Wait for acknowledgement
    await senderInteraction.interact(RELAY0, packet)

    await msgDefer.promise
  })
})
