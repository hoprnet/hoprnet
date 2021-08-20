import { Packet } from '../../messages'
import type HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'
import type PeerId from 'peer-id'
import { durations, pubKeyToPeerId, HoprDB } from '@hoprnet/hopr-utils'
import { Mixer } from '../../mixer'
import { sendAcknowledgement } from './acknowledgement'

const FORWARD_TIMEOUT = durations.seconds(6)

export class PacketForwardInteraction {
  private mixer: Mixer

  constructor(
    private subscribe: any,
    private sendMessage: any,
    private privKey: PeerId,
    private chain: HoprCoreEthereum,
    private emitMessage: (msg: Uint8Array) => void,
    private db: HoprDB,
    private protocolMsg: string,
    private protocolAck: string,
  ) {
    this.mixer = new Mixer(this.handleMixedPacket.bind(this))
    this.subscribe(protocolMsg, this.handlePacket.bind(this))
  }

  async interact(counterparty: PeerId, packet: Packet): Promise<void> {
    await this.sendMessage(counterparty, this.protocolMsg, packet.serialize(), {
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
      await packet.storeUnacknowledgedTicket(this.db)
      await packet.forwardTransform(this.privKey, this.chain)

      await this.interact(pubKeyToPeerId(packet.nextHop), packet)
    }

    sendAcknowledgement(packet, packet.previousHop.toPeerId(), this.sendMessage, this.privKey, this.protocolAck)
  }
}
