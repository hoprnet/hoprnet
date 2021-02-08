import { PROTOCOL_STRING } from '../../constants'
import { Packet } from '../../messages/packet'
import { Acknowledgement } from '../../messages/acknowledgement'

import Debug from 'debug'
const log = Debug('hopr-core:forward')
const verbose = Debug('hopr-core:verbose:forward')

import type PeerId from 'peer-id'

import type { AbstractInteraction } from '../abstractInteraction'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../'
import pipe from 'it-pipe'

import type { Connection, MuxedStream} from 'libp2p'
import { dialHelper, durations } from '@hoprnet/hopr-utils'
import { getTokens, Token } from '../../utils'
import { Mixer } from '../../mixer'

const MAX_PARALLEL_JOBS = 20
const FORWARD_TIMEOUT = durations.seconds(6)

class PacketForwardInteraction<Chain extends HoprCoreConnector> implements AbstractInteraction {
  private tokens: Token[] = getTokens(MAX_PARALLEL_JOBS)
  private promises: Promise<void>[] = []

  protocols: string[] = [PROTOCOL_STRING]

  constructor(public node: Hopr<Chain>, private mixer: Mixer<Chain>) {
    this.node._libp2p.handle(this.protocols, this.handler.bind(this))
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

  handler(struct: { connection: Connection, stream: MuxedStream, protocol: string }) {
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

          if (this.tokens.length > 0) {
            const token = this.tokens.pop() as Token

            this.promises[token] = this.handlePacket(token)
          }
        }
      }
    )
  }

  async handlePacket(token: number): Promise<void> {
    let packet: Packet<Chain>
    let sender: PeerId, target: PeerId

    while (this.mixer.notEmpty()) {
      if (this.mixer.poppable()) {
        try {
          packet = this.mixer.pop()

          const { receivedChallenge, ticketKey } = await packet.forwardTransform()

          ;[sender, target] = await Promise.all([packet.getSenderPeerId(), packet.getTargetPeerId()])

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
        } catch (error) {
          log('Error while handling packet')
          verbose('Error while handling packet', error)
        }
      } else {
        // Wait a bit for packet
        await new Promise((resolve) => setTimeout(resolve, this.mixer.WAIT_TIME))
      }
    }

    this.promises[token] = undefined
    this.tokens.push(token)
  }
}

export { PacketForwardInteraction }
