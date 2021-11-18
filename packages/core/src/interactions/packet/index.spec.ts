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
  ChannelEntry,
  ChannelStatus,
  privKeyToPeerId,
  stringToU8a,
  Hash,
  PRICE_PER_PACKET
} from '@hoprnet/hopr-utils'
import type { HalfKeyChallenge } from '@hoprnet/hopr-utils'
import assert from 'assert'
import { AcknowledgementChallenge, Packet, Acknowledgement } from '../../messages'
import { PacketForwardInteraction } from './forward'
import { initializeCommitment } from '@hoprnet/hopr-core-ethereum'
import { SECRET_LENGTH } from '@hoprnet/hopr-utils/lib/crypto/packet/constants'

//const SECRET_LENGTH = 32

const TEST_MESSAGE = new TextEncoder().encode('test message')

const DEFAULT_FUNDING = new Balance(new BN(1234).mul(PRICE_PER_PACKET))
const DEFAULT_TICKET_EPOCH = new UINT256(new BN(0))
const DEFAULT_INDEX = new UINT256(new BN(0))
const DEFAULT_CHANNEL_EPOCH = new UINT256(new BN(0))
const DEFAULT_CLOSURE_TIME = new UINT256(new BN(0))

const SELF = privKeyToPeerId(stringToU8a('0x492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775'))
const RELAY0 = privKeyToPeerId(stringToU8a('0x5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca'))
const RELAY1 = privKeyToPeerId(stringToU8a('0x3477d7de923ba3a7d5d72a7d6c43fd78395453532d03b2a1e2b9a7cc9b61bafa'))
const RELAY2 = privKeyToPeerId(stringToU8a('0xdb7e3e8fcac4c817aa4cecee1d6e2b4d53da51f9881592c0e1cc303d8a012b92'))
const COUNTERPARTY = privKeyToPeerId(stringToU8a('0x0726a9704d56a013980a9077d195520a61b5aed28f92d89c50bca6e0e0c48cfc'))

const nodes: PeerId[] = [SELF, RELAY0, RELAY1, RELAY2, COUNTERPARTY]

/**
 * Creates a mocked network to send and receive acknowledgements and packets
 * @param events simulates an Ethernet connection
 * @param self our own identity to know which messages are destined for us
 */
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

/**
 * Returns a channel entry that allows to issue
 * at least one ticket
 *
 * @dev might require changes if ticket validation changes
 *
 * @param from channel source
 * @param to channel destination
 * @returns channel representation
 */
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

/**
 * Opens mock channels and stores channel entry at source
 * and destination. Initializes the commitments for each channel
 * destination.
 * @param dbs node storage
 * @param nodes node identities
 */
async function createMinimalChannelTopology(dbs: HoprDB[], nodes: PeerId[]): Promise<void> {
  let previousChannel: ChannelEntry
  for (const [index, peerId] of nodes.entries()) {
    dbs[index] = HoprDB.createMock(PublicKey.fromPeerId(peerId))

    let channel: ChannelEntry

    if (index < nodes.length - 1) {
      channel = getDummyChannel(peerId, nodes[index + 1])

      // Store channel entry at source
      await dbs[index].updateChannel(channel.getId(), channel)
    }

    if (index > 0) {
      // Store channel entry at destination
      await dbs[index].updateChannel(previousChannel.getId(), previousChannel)

      // Set a commitment if we are the destination
      await initializeCommitment(
        dbs[index],
        previousChannel.getId(),
        (): any => {},
        (): any => {}
      )
    }

    previousChannel = channel
  }
}

class TestingForwardInteraction extends PacketForwardInteraction {
  /**
   * Disable probablistic mixing
   */
  useMockMixer() {
    const push = (p: Packet) => {
      this.handleMixedPacket(p)
    }
    this.mixer = {
      push
    } as any
  }
}

// Tests two different packet acknowledgement settings
// - Acknowledgement for packet sender
// - Acknowledgement for relayer, unlocking a ticket
describe('packet acknowledgement', function () {
  // Creating commitment chain takes time ...
  this.timeout(20e3)

  const events = new EventEmitter()
  let dbs: HoprDB[] = Array.from({ length: nodes.length }, (_, index) =>
    HoprDB.createMock(PublicKey.fromPeerId(nodes[index]))
  )

  afterEach(async function () {
    events.removeAllListeners()

    await Promise.all(dbs.map((db: HoprDB) => db.close))
  })

  beforeEach(async function () {
    await createMinimalChannelTopology(dbs, nodes)
  })

  // We create a packet and send it to the first relayer.
  // The first relayer receives it and sends an acknowledgement.
  // The acknowledgement *must* be received by the sender.
  // Despite it is not useful, the sender *must* understand it
  // and call `onMessage`.
  it('acknowledgement workflow as sender', async function () {
    const secrets: Uint8Array[] = Array.from({ length: 2 }, () => Uint8Array.from(randomBytes(SECRET_LENGTH)))

    const { ackChallenge } = createPoRValuesForSender(secrets[0], secrets[1])

    const libp2pSelf = createFakeSendReceive(events, SELF)
    const libp2pCounterparty = createFakeSendReceive(events, COUNTERPARTY)

    const ackReceived = defer<void>()

    await dbs[0].storePendingAcknowledgement(ackChallenge, true)

    subscribeToAcknowledgements(
      libp2pSelf.subscribe,
      dbs[0],
      SELF,
      (receivedAckChallenge: HalfKeyChallenge) => {
        if (receivedAckChallenge.eq(ackChallenge)) {
          ackReceived.resolve()
        }
      },
      () => {},
      () => {},
      'protocolAck'
    )

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
      libp2pCounterparty.send as any,
      COUNTERPARTY,
      'protocolAck'
    )

    await ackReceived.promise
  })

  // We receive a packet, run the transformation, extract keys
  // and validate the ticket.
  // Then we use the private key of the next downstream node to
  // extract the shared key and send an acknowledgement.
  // The acknowledgement *must* be received and the half key
  // *must* be sufficient to solve the challenge.
  it('acknowledgement workflow as relayer', async function () {
    const packet = await Packet.create(TEST_MESSAGE, [RELAY0, COUNTERPARTY], SELF, dbs[0])

    const libp2pRelay0 = createFakeSendReceive(events, RELAY0)

    const ackReceived = defer<void>()

    subscribeToAcknowledgements(
      libp2pRelay0.subscribe,
      dbs[1],
      RELAY0,
      () => {},
      () => {
        ackReceived.resolve()
      },
      () => {},
      'protocolAck'
    )

    const interaction = new TestingForwardInteraction(
      libp2pRelay0.subscribe,
      libp2pRelay0.send as any,
      RELAY0,
      () => {
        throw Error(`Node is not supposed to receive message`)
      },
      dbs[1],
      'protocolMsg',
      'protocolAck'
    )

    interaction.useMockMixer()

    const libp2pCounterparty = createFakeSendReceive(events, COUNTERPARTY)

    libp2pCounterparty.subscribe('protocolMsg', (msg: Uint8Array) => {
      sendAcknowledgement(
        Packet.deserialize(msg, COUNTERPARTY, RELAY0),
        RELAY0,
        libp2pCounterparty.send as any,
        COUNTERPARTY,
        'protocolAck'
      )
    })

    interaction.handleMixedPacket(Packet.deserialize(packet.serialize(), RELAY0, SELF))

    await ackReceived.promise
  })
})

// Integration test:
// Creates a multi-hop packet, and sends it along the path while
// using the packet forwarding code.
describe('packet relaying interaction', function () {
  // Creating commitment chain takes time ...
  this.timeout(20e3)

  const events = new EventEmitter()
  let dbs: HoprDB[] = Array.from({ length: nodes.length }, (_, index) =>
    HoprDB.createMock(PublicKey.fromPeerId(nodes[index]))
  )

  afterEach(async function () {
    events.removeAllListeners()

    await Promise.all(dbs.map((db: HoprDB) => db.close))
  })

  beforeEach(async function () {
    await createMinimalChannelTopology(dbs, nodes)
  })

  it('packet-acknowledgement multi-relay workflow', async function () {
    const msgDefer = defer<void>()
    const nodes: PeerId[] = [SELF, RELAY0, RELAY1, RELAY2, COUNTERPARTY]

    let senderInteraction: TestingForwardInteraction

    for (const [index, pId] of nodes.entries()) {
      const { subscribe, send } = createFakeSendReceive(events, pId)

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

      const interaction = new TestingForwardInteraction(
        subscribe,
        send as any,
        pId,
        receive,
        dbs[index],
        'protocolMsg',
        'protocolAck'
      )
      interaction.useMockMixer()

      if (pId.equals(SELF)) {
        senderInteraction = interaction
      }
    }

    const packet = await Packet.create(TEST_MESSAGE, [RELAY0, RELAY1, RELAY2, COUNTERPARTY], SELF, dbs[0])

    await packet.storePendingAcknowledgement(dbs[0])

    // Wait for acknowledgement
    await senderInteraction.interact(RELAY0, packet)

    await msgDefer.promise
  })
})
