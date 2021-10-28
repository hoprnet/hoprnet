import { PROTOCOL_STRING } from '../../constants'
import { Packet } from '../../messages'
import type PeerId from 'peer-id'
import { durations, pubKeyToPeerId, HoprDB } from '@hoprnet/hopr-utils'
import { Mixer } from '../../mixer'
import { sendAcknowledgement } from './acknowledgement'
import { debug } from '@hoprnet/hopr-utils'
import type { SendMessage } from '../../index'

const log = debug('hopr-core:packet:forward')

const FORWARD_TIMEOUT = durations.seconds(6)

export class PacketForwardInteraction {
  private mixer: Mixer

  constructor(
    private subscribe: any,
    private sendMessage: SendMessage,
    private privKey: PeerId,
    private emitMessage: (msg: Uint8Array) => void,
    private db: HoprDB
  ) {
    this.mixer = new Mixer(this.handleMixedPacket.bind(this))
    this.subscribe(PROTOCOL_STRING, this.handlePacket.bind(this))
  }

  async interact(counterparty: PeerId, packet: Packet): Promise<void> {
    await this.sendMessage(counterparty, PROTOCOL_STRING, packet.serialize(), false, {
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
    } else {
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
    }

    sendAcknowledgement(packet, packet.previousHop.toPeerId(), this.sendMessage, this.privKey)
  }
}
