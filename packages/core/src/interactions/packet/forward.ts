import { PROTOCOL_STRING } from '../../constants'
import { Packet } from '../../messages/packet'
import { AcknowledgementMessage } from '../../messages/acknowledgement'
import { Logger } from '@hoprnet/hopr-utils'
import type PeerId from 'peer-id'
import { durations, oneAtATime, HoprDB } from '@hoprnet/hopr-utils'
import { Mixer } from '../../mixer'
import { Challenge } from '../../messages/packet/challenge'
import { PROTOCOL_ACKNOWLEDGEMENT } from '../../constants'
import LibP2P from 'libp2p'
import HoprCoreEthereum from '@hoprnet/hopr-core-ethereum'

const log = Logger.getLogger('hopr-core.forward')
const FORWARD_TIMEOUT = durations.seconds(6)
const ACKNOWLEDGEMENT_TIMEOUT = durations.seconds(2)

class PacketForwardInteraction {
  private mixer: Mixer
  private concurrencyLimiter

  constructor(
    private db: HoprDB,
    private paymentChannels: HoprCoreEthereum,
    private id: PeerId,
    private libp2p: LibP2P,
    private subscribe: any,
    private sendMessage: any,
    private onMessage: (msg: Uint8Array) => void
  ) {
    this.mixer = new Mixer(this.handleMixedPacket.bind(this))
    this.concurrencyLimiter = oneAtATime()
    this.subscribe(PROTOCOL_STRING, this.handlePacket.bind(this))
  }

  async interact(counterparty: PeerId, packet: Packet): Promise<void> {
    await this.sendMessage(counterparty, PROTOCOL_STRING, packet, {
      timeout: FORWARD_TIMEOUT
    })
  }

  async handlePacket(msg: Uint8Array) {
    const arr = msg.slice()
    const packet = new Packet(this.libp2p, this.paymentChannels, this.db, this.id, {
      bytes: arr.buffer,
      offset: arr.byteOffset
    })

    this.mixer.push(packet)
  }

  async handleMixedPacket(packet: Packet) {
    const { id, onMessage, sendMessage } = this
    const interact = this.interact.bind(this)
    this.concurrencyLimiter(async function () {
      // See discussion in #1256 - apparently packet.forwardTransform cannot be
      // called concurrently
      try {
        const { receivedChallenge, ticketKey } = await packet.forwardTransform()
        const [sender, target] = await Promise.all([packet.getSenderPeerId(), packet.getTargetPeerId()])

        setImmediate(async () => {
          const ack = await AcknowledgementMessage.create(Challenge.deserialize(ticketKey), receivedChallenge, id)
          sendMessage(sender, PROTOCOL_ACKNOWLEDGEMENT, ack.serialize(), {
            timeout: ACKNOWLEDGEMENT_TIMEOUT
          })
        })

        if (id.equals(target)) {
          onMessage(packet.message.plaintext)
        } else {
          await interact(target, packet)
        }
      } catch (error) {
        log.error('Error while handling packet', error)
      }
    })
  }
}

export { PacketForwardInteraction }
