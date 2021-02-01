import { PROTOCOL_STRING } from '../../constants'
import { Packet } from '../../messages/packet'
import { Acknowledgement } from '../../messages/acknowledgement'

import Debug from 'debug'
const log = Debug('hopr-core:forward')

import type PeerId from 'peer-id'

import type { AbstractInteraction } from '../abstractInteraction'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../'
import pipe from 'it-pipe'

import type { Handler } from 'libp2p'

import { dialHelper, durations } from '@hoprnet/hopr-utils'
import { Mixer } from '../../mixer'

const FORWARD_TIMEOUT = durations.seconds(6)

class PacketForwardInteraction<Chain extends HoprCoreConnector> implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_STRING]

  constructor(public node: Hopr<Chain>, private mixer: Mixer<Chain>) {
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))

    this.drainMixer()
  }

  async drainMixer() {
    for await (const packet of this.mixer[Symbol.asyncIterator]()) {
      // @TODO add try / catch block
      log(`Got packet from mixer. Mixer has another ${this.mixer.length} packets.`)
      const { receivedChallenge, ticketKey } = await packet.forwardTransform()

      const [sender, target] = await Promise.all([packet.getSenderPeerId(), packet.getTargetPeerId()])

      setImmediate(async () => {
        const ack = new Acknowledgement(this.node.paymentChannels, undefined, {
          key: ticketKey,
          challenge: receivedChallenge
        })

        await this.node._interactions.packet.acknowledgment.interact(sender, await ack.sign(this.node.getId()))
      })

      if (this.node.getId().equals(target)) {
        this.node.output(packet.message.plaintext)
      } else {
        await this.interact(target, packet)
      }
    }
  }

  async interact(counterparty: PeerId, packet: Packet<Chain>): Promise<void> {
    const struct = await dialHelper(this.node._libp2p, counterparty, this.protocols, {
      timeout: FORWARD_TIMEOUT
    })

    if (struct == undefined) {
      throw Error(`Failed to send packet to ${counterparty.toB58String()}.`)
    }

    pipe([packet], struct.stream)
  }

  handler(struct: Handler): void {
    pipe(
      /* pretttier-ignore */
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
}

export { PacketForwardInteraction }
