import { PROTOCOL_STRING } from '../../constants'
import { Packet } from '../../messages/packet'
import { AcknowledgementMessage } from '../../messages/acknowledgement'
import Debug from 'debug'
import type PeerId from 'peer-id'
import type { AbstractInteraction } from '../abstractInteraction'
import type Hopr from '../../'
import pipe from 'it-pipe'
import type { Connection, MuxedStream } from 'libp2p'
import { dialHelper, durations, oneAtATime } from '@hoprnet/hopr-utils'
import { Mixer } from '../../mixer'
import { Challenge } from '../../messages/packet/challenge'

const log = Debug('hopr-core:forward')
const FORWARD_TIMEOUT = durations.seconds(6)

class PacketForwardInteraction implements AbstractInteraction {
  private mixer: Mixer
  private concurrencyLimiter
  protocols: string[] = [PROTOCOL_STRING]

  constructor(public node: Hopr) {
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))
    this.mixer = new Mixer(this.handleMixedPacket.bind(this))
    this.concurrencyLimiter = oneAtATime()
  }

  async interact(counterparty: PeerId, packet: Packet): Promise<void> {
    const struct = await dialHelper(this.node._libp2p, counterparty, this.protocols[0], {
      timeout: FORWARD_TIMEOUT
    })

    if (struct == undefined) {
      throw Error(`Failed to send packet to ${counterparty.toB58String()}.`)
    }

    pipe([packet], struct.stream)
  }

  handler(struct: { connection: Connection; stream: MuxedStream; protocol: string }) {
    pipe(
      struct.stream,
      async (source: AsyncIterable<Uint8Array>): Promise<void> => {
        for await (const msg of source) {
          const arr = msg.slice()
          const packet = new Packet(this.node, this.node._libp2p, {
            bytes: arr.buffer,
            offset: arr.byteOffset
          })

          this.mixer.push(packet)
        }
      }
    )
  }

  async handleMixedPacket(packet: Packet) {
    const node = this.node
    const interact = this.interact.bind(this)
    this.concurrencyLimiter(async function () {
      // See discussion in #1256 - apparently packet.forwardTransform cannot be
      // called concurrently
      try {
        const { receivedChallenge, ticketKey } = await packet.forwardTransform()
        const [sender, target] = await Promise.all([packet.getSenderPeerId(), packet.getTargetPeerId()])

        setImmediate(async () => {
          const ack = await AcknowledgementMessage.create(
            Challenge.deserialize(ticketKey),
            receivedChallenge,
            node.getId()
          )
          await node._interactions.packet.acknowledgment.interact(sender, ack)
        })

        if (node.getId().equals(target)) {
          node.output(packet.message.plaintext)
        } else {
          await interact(target, packet)
        }
      } catch (error) {
        log('Error while handling packet', error)
      }
    })
  }
}

export { PacketForwardInteraction }
