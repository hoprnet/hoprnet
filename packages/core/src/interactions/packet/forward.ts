import { setImmediate } from 'timers/promises'

import type { PeerId } from '@libp2p/interface-peer-id'

import { durations, pubKeyToPeerId, HoprDB } from '@hoprnet/hopr-utils'
import { debug } from '@hoprnet/hopr-utils'

import { Packet } from '../../messages/index.js'
import { Mixer } from '../../mixer.js'
import { sendAcknowledgement } from './acknowledgement.js'
import type { SendMessage, Subscribe } from '../../index.js'

const log = debug('hopr-core:packet:forward')
const error = debug('hopr-core:packet:forward:error')

const FORWARD_TIMEOUT = durations.seconds(6)

export class PacketForwardInteraction {
  protected mixer: Mixer

  constructor(
    private subscribe: Subscribe,
    private sendMessage: SendMessage,
    private privKey: PeerId,
    private emitMessage: (msg: Uint8Array) => void,
    private db: HoprDB,
    private protocolMsg: string | string[],
    private protocolAck: string | string[]
  ) {
    this.mixer = new Mixer(this.handleMixedPacket.bind(this))
  }

  private errHandler(err: any) {
    error(`Error while receiving packet`, err)
  }

  async start() {
    await this.subscribe(this.protocolMsg, this.handlePacket.bind(this), false, this.errHandler)
  }

  async interact(counterparty: PeerId, packet: Packet): Promise<void> {
    await this.sendMessage(counterparty, this.protocolMsg, packet.serialize(), false, {
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
      // defer processing to end of event loop since we are making another
      // network operation
      await setImmediate()
      await sendAcknowledgement(packet, packet.previousHop.toPeerId(), this.sendMessage, this.privKey, this.protocolAck)
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

    // defer processing to end of event loop since we are making another
    // network operation
    await setImmediate()
    await sendAcknowledgement(packet, packet.previousHop.toPeerId(), this.sendMessage, this.privKey, this.protocolAck)
  }
}
