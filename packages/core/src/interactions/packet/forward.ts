import { Packet } from '../../messages'
import type PeerId from 'peer-id'
import { durations, pubKeyToPeerId, HoprDB } from '@hoprnet/hopr-utils'
import { Mixer } from '../../mixer'
import { sendAcknowledgement } from './acknowledgement'
import { debug } from '@hoprnet/hopr-utils'
import type { SendMessage, Subscribe } from '../../index'

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
    private protocolMsg: string,
    private protocolAck: string
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
      sendAcknowledgement(packet, packet.previousHop.toPeerId(), this.sendMessage, this.privKey, this.protocolAck)
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

    sendAcknowledgement(packet, packet.previousHop.toPeerId(), this.sendMessage, this.privKey, this.protocolAck)
  }
}
