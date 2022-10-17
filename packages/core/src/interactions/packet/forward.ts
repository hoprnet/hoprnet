import type { PeerId } from '@libp2p/interface-peer-id'

import { durations, pickVersion, pubKeyToPeerId, type HoprDB } from '@hoprnet/hopr-utils'
import { debug } from '@hoprnet/hopr-utils'

import { Packet } from '../../messages/index.js'
import { Mixer } from '../../mixer.js'
import type { AcknowledgementInteraction } from './acknowledgement.js'
import type { SendMessage, Subscribe } from '../../index.js'
import type { ResolvedEnvironment } from '../../environment.js'

const log = debug('hopr-core:packet:forward')
const error = debug('hopr-core:packet:forward:error')

const FORWARD_TIMEOUT = durations.seconds(6)

// Do not type-check JSON files
// @ts-ignore
import pkg from '../../../package.json' assert { type: 'json' }

const NORMALIZED_VERSION = pickVersion(pkg.version)

export class PacketForwardInteraction {
  protected mixer: Mixer

  public readonly protocols: string | string[]

  constructor(
    private subscribe: Subscribe,
    private sendMessage: SendMessage,
    private privKey: PeerId,
    private emitMessage: (msg: Uint8Array) => void,
    private db: HoprDB,
    private environment: ResolvedEnvironment,
    private acknowledgements: AcknowledgementInteraction,
    // used for testing
    nextRandomInt?: () => number
  ) {
    this.mixer = new Mixer(nextRandomInt)
    this.handlePacket = this.handlePacket.bind(this)

    this.protocols = [
      // current
      `/hopr/${this.environment.id}/msg/${NORMALIZED_VERSION}`,
      // deprecated
      `/hopr/${this.environment.id}/msg`
    ]
  }

  private errHandler(err: any) {
    error(`Error while receiving packet`, err)
  }

  async start() {
    await this.subscribe(this.protocols, this.handlePacket, false, this.errHandler)

    this.handleMixedPackets()
  }

  stop() {
    // Clear mixer timeouts
    this.mixer.end()
  }

  async handleMixedPackets() {
    for await (const packet of this.mixer) {
      await this.handleMixedPacket(packet)
    }
  }

  async interact(counterparty: PeerId, packet: Packet): Promise<void> {
    await this.sendMessage(counterparty, this.protocols, packet.serialize(), false, {
      timeout: FORWARD_TIMEOUT
    })
  }

  async handlePacket(msg: Uint8Array, remotePeer: PeerId) {
    const packet = Packet.deserialize(msg, this.privKey, remotePeer)

    this.mixer.push(packet)
  }

  async handleMixedPacket(packet: Packet) {
    await packet.checkPacketTag(this.db)

    if (packet.isReceiver) {
      this.emitMessage(packet.plaintext)
      // Send acknowledgements independently
      this.acknowledgements.sendAcknowledgement(packet, packet.previousHop.toPeerId())
      // Nothing else to do
      return
    }

    // Packet should be forwarded
    try {
      await packet.validateUnacknowledgedTicket(this.db)
    } catch (err) {
      log(`Ticket validation failed. Dropping packet`, err)
      return
    }

    await packet.storeUnacknowledgedTicket(this.db)

    try {
      await packet.forwardTransform(this.privKey, this.db)
    } catch (err) {
      log(`Packet transformation failed. Dropping packet`, err)
      return
    }

    try {
      await this.interact(pubKeyToPeerId(packet.nextHop), packet)
    } catch (err) {
      log(`Forwarding transformed packet failed.`, err)
      return
    }

    // Send acknowledgements independently
    this.acknowledgements.sendAcknowledgement(packet, packet.previousHop.toPeerId())
  }
}
