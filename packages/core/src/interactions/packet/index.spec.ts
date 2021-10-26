import type PeerId from 'peer-id'
import { randomBytes } from 'crypto'
import { EventEmitter } from 'events'
import BN from 'bn.js'

import { subscribeToAcknowledgements, sendAcknowledgement } from './acknowledgement'
import {
  Balance,
  defer,
  HoprDB,
  PublicKey,
  UINT256,
  createPoRValuesForSender,
  deriveAckKeyShare,
  u8aEquals,
  privKeyToPeerId,
  stringToU8a,
  ChannelEntry,
  Hash,
  ChannelStatus
} from '@hoprnet/hopr-utils'
import assert from 'assert'
import { PROTOCOL_STRING } from '../../constants'
import { AcknowledgementChallenge, Packet, Acknowledgement } from '../../messages'
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

function getDummyChannel(from: PeerId, to: PeerId): ChannelEntry {
  return new ChannelEntry(
    PublicKey.fromPeerId(from),
    PublicKey.fromPeerId(to),
    DEFAULT_FUNDING,
    new Hash(Uint8Array.from(randomBytes(32))),
    DEFAULT_TICKET_EPOCH,
    DEFAULT_INDEX,
    ChannelStatus.Open,
    DEFAULT_CHANNEL_EPOCH,
    DEFAULT_CLOSURE_TIME
  )
}

describe('packet interaction', function () {
  const events = new EventEmitter()
  let dbs: HoprDB[] = Array.from({ length: nodes.length })

  afterEach(async function () {
    events.removeAllListeners()

    await Promise.all(dbs.map((db: HoprDB) => db.close))
  })

  beforeEach(function () {
    for (const [index, node] of nodes.entries()) {
      dbs[index] = HoprDB.createMock(PublicKey.fromPeerId(node))
    }
  })

  it('acknowledgement workflow as sender', async function () {
    const secrets: Uint8Array[] = Array.from({ length: 2 }, () => Uint8Array.from(randomBytes(SECRET_LENGTH)))

    const { ackChallenge } = createPoRValuesForSender(secrets[0], secrets[1])

    const libp2pSelf = createFakeSendReceive(events, SELF)
    const libp2pCounterparty = createFakeSendReceive(events, COUNTERPARTY)

    const ackReceived = defer<void>()

    subscribeToAcknowledgements(libp2pSelf.subscribe, dbs[0], new EventEmitter(), SELF, () => ackReceived.resolve())

    const ackKey = deriveAckKeyShare(secrets[0])
    const ackMessage = AcknowledgementChallenge.create(ackChallenge, SELF)

    assert(
      ackMessage.solve(ackKey.serialize()),
      `acknowledgement key must be sufficient to solve acknowledgement challenge`
    )

    sendAcknowledgement(
      {
        createAcknowledgement: (privKey: PeerId) => {
          return Acknowledgement.create(ackMessage, ackKey, privKey)
        }
      } as any,
      SELF,
      libp2pCounterparty.send,
      COUNTERPARTY
    )

    await ackReceived.promise
  })

  it('acknowledgement workflow as relayer', async function () {
    // Open a dummy channel to create first packet
    const firstChannel = getDummyChannel(SELF, RELAY0)
    await dbs[0].updateChannel(firstChannel.getId(), firstChannel)

    const packet = await Packet.create(TEST_MESSAGE, [RELAY0, COUNTERPARTY], SELF, dbs[0])

    const libp2pRelay0 = createFakeSendReceive(events, RELAY0)

    const ackReceived = defer<void>()

    subscribeToAcknowledgements(libp2pRelay0.subscribe, dbs[1], new EventEmitter(), RELAY0, () => ackReceived.resolve())

    const interaction = new PacketForwardInteraction(
      libp2pRelay0.subscribe,
      libp2pRelay0.send,
      RELAY0,
      () => {
        throw Error(`Node is not supposed to receive message`)
      },
      dbs[1]
    )

    const libp2pCounterparty = createFakeSendReceive(events, COUNTERPARTY)

    libp2pCounterparty.subscribe(PROTOCOL_STRING, (msg: Uint8Array) => {
      sendAcknowledgement(Packet.deserialize(msg, COUNTERPARTY, RELAY0), RELAY0, libp2pCounterparty.send, COUNTERPARTY)
    })

    interaction.handleMixedPacket(Packet.deserialize(packet.serialize(), RELAY0, SELF))

    await ackReceived.promise
  })

  it('packet-acknowledgement multi-relay workflow', async function () {
    const msgDefer = defer<void>()
    const nodes: PeerId[] = [SELF, RELAY0, RELAY1, RELAY2, COUNTERPARTY]

    let senderInteraction: PacketForwardInteraction

    for (const [index, pId] of nodes.entries()) {
      const { subscribe, send } = createFakeSendReceive(events, pId)

      if (!pId.equals(RELAY2) && !pId.equals(COUNTERPARTY)) {
        // Open dummy channels to issue tickets
        const channel = getDummyChannel(pId, nodes[index + 1])
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
