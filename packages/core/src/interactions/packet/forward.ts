import type { PeerId } from '@libp2p/interface-peer-id'

import { durations, pickVersion, type HoprDB, create_counter } from '@hoprnet/hopr-utils'
import { debug, registerMetricsCollector } from '@hoprnet/hopr-utils'

import { pushable, type Pushable } from 'it-pushable'

import { Packet, PacketHelper, PacketState } from '../../messages/index.js'
import { new_mixer, core_mixer_initialize_crate, core_mixer_gather_metrics } from '../../../lib/core_mixer.js'
core_mixer_initialize_crate()
registerMetricsCollector(core_mixer_gather_metrics)

import type { AcknowledgementInteraction } from './acknowledgement.js'
import type { HoprOptions, SendMessage } from '../../index.js'
import type { ResolvedNetwork } from '../../network.js'
import type { Components } from '@libp2p/interfaces/components'

const log = debug('hopr-core:packet:forward')
const error = debug('hopr-core:packet:forward:error')

const FORWARD_TIMEOUT = durations.seconds(6)

// Metrics
const metric_fwdMessageCount = create_counter('core_counter_forwarded_messages', 'Number of forwarded messages')
const metric_recvMessageCount = create_counter('core_counter_received_messages', 'Number of received messages')

// Do not type-check JSON files
// @ts-ignore
import pkg from '../../../package.json' assert { type: 'json' }
import { peerIdFromBytes, peerIdFromString } from '@libp2p/peer-id'
import { keysPBM } from '@libp2p/crypto/keys'

const NORMALIZED_VERSION = pickVersion(pkg.version)

export class PacketForwardInteraction {
  protected packetQueue: Pushable<Uint8Array>

  public readonly protocols: string | string[]

  constructor(
    private libp2pComponents: Components,
    private sendMessage: SendMessage,
    private privKey: PeerId,
    private emitMessage: (msg: Uint8Array) => void,
    private db: HoprDB,
    private network: ResolvedNetwork,
    private acknowledgements: AcknowledgementInteraction,
    private options: HoprOptions
  ) {
    this.packetQueue = pushable()

    this.protocols = [
      // current
      `/hopr/${this.network.id}/msg/${NORMALIZED_VERSION}`,
      // deprecated
      `/hopr/${this.network.id}/msg`
    ]
  }

  private errHandler(err: any) {
    error(`Error while receiving packet`, err)
  }

  async start() {
    await this.libp2pComponents.getRegistrar().handle(this.protocols, async ({ connection, stream }) => {
      try {
        for await (const chunk of stream.source) {
          // TODO: this is a temporary quick-and-dirty solution to be used until
          // packet transformation logic has been ported to Rust
          this.packetQueue.push(Uint8Array.from([...connection.remotePeer.toBytes(), ...chunk]))
        }
      } catch (err) {
        this.errHandler(err)
      }
    })

    this.handleMixedPackets()
  }

  stop() {
    this.packetQueue.end()
  }

  async handleMixedPackets() {
    let self = this
    for await (const chunk of {
      [Symbol.asyncIterator]() {
        return new_mixer(self.packetQueue[Symbol.asyncIterator]())
      }
    }) {
      // TODO: this is a temporary quick-and-dirty solution to be used until
      // packet transformation logic has been ported to Rust
      const sender = peerIdFromBytes(chunk.slice(0, 39))
      let private_key = keysPBM.PrivateKey.decode(this.privKey.privateKey).Data
      const packet = Packet.deserialize(chunk.slice(39), private_key, sender.toString())

      await this.handleMixedPacket(packet)
    }
  }

  async interact(counterparty: PeerId, packet: Packet): Promise<void> {
    await this.sendMessage(counterparty, this.protocols, packet.serialize(), false, {
      timeout: FORWARD_TIMEOUT
    })
  }

  async handleMixedPacket(packet: Packet) {
    await PacketHelper.checkPacketTag(packet, this.db)

    if (packet.state() == PacketState.Final) {
      this.emitMessage(packet.plaintext())
      // Send acknowledgements independently
      this.acknowledgements.sendAcknowledgement(packet, peerIdFromString(packet.previous_hop().to_peerid_str()))
      metric_recvMessageCount.increment()
      // Nothing else to do
      return
    }

    // Packet should be forwarded
    try {
      await PacketHelper.validateUnacknowledgedTicket(packet, this.db, this.options.checkUnrealizedBalance)
    } catch (err) {
      log(`Ticket validation failed. Dropping packet`, err)
      return
    }

    await PacketHelper.storeUnacknowledgedTicket(packet, this.db)

    try {
      await PacketHelper.forwardTransform(packet, this.privKey, this.db)
    } catch (err) {
      log(`Packet transformation failed. Dropping packet`, err)
      return
    }

    try {
      await this.interact(peerIdFromString(packet.next_hop().to_peerid_str()), packet)
    } catch (err) {
      log(`Forwarding transformed packet failed.`, err)
      return
    }

    // Send acknowledgements independently
    this.acknowledgements.sendAcknowledgement(packet, peerIdFromString(packet.previous_hop().to_peerid_str()))
    metric_fwdMessageCount.increment()
  }
}
