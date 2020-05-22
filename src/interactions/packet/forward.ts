import { PROTOCOL_STRING } from '../../constants'
import { Packet } from '../../messages/packet'
import { Acknowledgement } from '../../messages/acknowledgement'

import type PeerId from 'peer-id'
import PeerInfo from 'peer-info'
import chalk from 'chalk'

import AbortController from 'abort-controller'

import type { AbstractInteraction } from '../abstractInteraction'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '../../'
import pipe from 'it-pipe'

import type { Handler } from '../../network/transport/types'

import { randomInteger } from '@hoprnet/hopr-utils'
import { getTokens, Token } from '../../utils'

const MAX_PARALLEL_JOBS = 20

const TWO_SECONDS = 2 * 1000
const FORWARD_TIMEOUT = TWO_SECONDS

class PacketForwardInteraction<Chain extends HoprCoreConnector>
  implements AbstractInteraction<Chain> {
  private tokens: Token[] = getTokens(MAX_PARALLEL_JOBS)
  private queue: Packet<Chain>[] = []
  private promises: Promise<void>[] = []

  protocols: string[] = [PROTOCOL_STRING]

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  async interact(counterparty: PeerInfo | PeerId, packet: Packet<Chain>): Promise<void> {
    let struct: Handler

    const abort = new AbortController()
    const signal = abort.signal

    const timeout = setTimeout(() => {
      abort.abort()
    }, FORWARD_TIMEOUT)

    struct = await this.node
      .dialProtocol(counterparty, this.protocols[0], { signal })
      .catch(async (err: Error) => {
        const peerInfo = await this.node.peerRouting.findPeer(
          PeerInfo.isPeerInfo(counterparty) ? counterparty.id : counterparty
        )

        try {
          let result = await this.node.dialProtocol(peerInfo, this.protocols[0], { signal })
          clearTimeout(timeout)
          return result
        } catch (err) {
          clearTimeout(timeout)

          this.node.log(
            `Could not transfer packet to ${(PeerInfo.isPeerInfo(counterparty)
              ? counterparty.id
              : counterparty
            ).toB58String()}. Error was: ${chalk.red(err.message)}.`
          )

          return
        }
      })

    await pipe(
      /* prettier-ignore */
      [packet.subarray()],
      struct.stream
    )
  }

  handler(struct: Handler): void {
    let packet: Packet<Chain>
    pipe(
      /* pretttier-ignore */
      struct.stream,
      async (source: AsyncIterable<Uint8Array>): Promise<void> => {
        for await (const msg of source) {
          const arr = msg.slice()
          packet = new Packet(this.node, {
            bytes: arr.buffer,
            offset: arr.byteOffset,
          })

          this.queue.push(packet)

          if (this.tokens.length > 0) {
            const token = this.tokens.pop() as Token
            if (this.promises[token] != null) {
              /**
               * @TODO remove this and make sure that the Promise is always
               * already resolved.
               */
              await this.promises[token]

              this.promises[token] = this.handlePacket(token)
            } else {
              this.handlePacket(token)
            }
          }
        }
      }
    )
  }

  async handlePacket(token: number): Promise<void> {
    let packet: Packet<Chain>
    let sender: PeerId, target: PeerId

    // Check for unserviced packets
    while (this.queue.length > 0) {
      // Pick a random one
      const index = randomInteger(0, this.queue.length)

      if (index == this.queue.length - 1) {
        packet = this.queue.pop() as Packet<Chain>
      } else {
        packet = this.queue[index]

        this.queue[index] = this.queue.pop() as Packet<Chain>
      }

      let { receivedChallenge, ticketKey } = await packet.forwardTransform()

      /* prettier-ignore */
      ;[sender, target] = await Promise.all([
        /* prettier-ignore */
        packet.getSenderPeerId(),
        packet.getTargetPeerId(),
      ])

      setImmediate(async () => {
        const ack = new Acknowledgement(this.node.paymentChannels, undefined, {
          key: ticketKey,
          challenge: receivedChallenge,
        })

        await this.node.interactions.packet.acknowledgment.interact(
          sender,
          await ack.sign(this.node.peerInfo.id)
        )
      })

      if (this.node.peerInfo.id.isEqual(target)) {
        this.node.output(packet.message.plaintext)
      } else {
        await this.interact(target, packet)
      }
    }

    this.tokens.push(token)
  }
}

export { PacketForwardInteraction }
