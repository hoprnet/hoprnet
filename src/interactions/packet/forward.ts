'use strict'

const Queue = require('promise-queue')

import { PROTOCOL_STRING, MAX_HOPS } from '../../constants'
import { Packet } from '../../messages/packet'
import { Acknowledgement } from '../../messages/acknowledgement'

import PeerId from 'peer-id'
import PeerInfo from 'peer-info'
import chalk from 'chalk'

import { AbstractInteraction } from '../abstractInteraction'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'
import Hopr from '../../'
import pipe from 'it-pipe'

import { deriveTicketKeyBlinding } from '../../messages/packet/header'

import { getTokens, Token, randomInteger } from '../../utils'

const MAX_PARALLEL_JOBS = 20

class PacketForwardInteraction<Chain extends HoprCoreConnectorInstance> implements AbstractInteraction<Chain> {
  private tokens: Token[] = getTokens(MAX_PARALLEL_JOBS)
  private queue: Packet<Chain>[] = []
  private promises: Promise<void>[] = []

  protocols: string[] = [PROTOCOL_STRING]

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  async interact(counterparty: PeerInfo | PeerId, packet: Packet<Chain>): Promise<void> {
    let struct: {
      stream: any
      protocol: string
    }

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0]).catch(async (err: Error) => {
        return this.node.peerRouting
          .findPeer(PeerInfo.isPeerInfo(counterparty) ? counterparty.id : counterparty)
          .then((peerInfo: PeerInfo) => this.node.dialProtocol(peerInfo, this.protocols[0]))
      })
    } catch (err) {
      this.node.log(
        `Could not transfer packet to ${(PeerInfo.isPeerInfo(counterparty) ? counterparty.id : counterparty).toB58String()}. Error was: ${chalk.red(
          err.message
        )}.`
      )

      return
    }

    return pipe(
      /* prettier-ignore */
      [packet],
      struct.stream
    )
  }

  handler(struct: { stream: any }): void {
    pipe(
      /* pretttier-ignore */
      struct.stream,
      async (source: AsyncIterable<Uint8Array>): Promise<void> => {
        for await (const msg of source) {
          const packet = new Packet(this.node, msg)

          if (this.tokens.length > 0) {
            const token = this.tokens.pop()
            if (this.promises.length == MAX_PARALLEL_JOBS) {
              /**
               * @TODO remove this and make sure that the Promise is always
               * already resolved.
               */
              await this.promises[token]

              this.promises[token] = this.handlePacket(packet, token)
            } else if (this.promises.length < MAX_PARALLEL_JOBS) {
              this.handlePacket(packet, this.tokens.pop())
            } else {
              throw Error(`something went wrong. Concurrency issue.`)
            }
          } else {
            this.queue.push(packet)
          }
        }
      }
    )
  }

  async handlePacket(packet: Packet<Chain>, token: number): Promise<void> {
    const sender = await packet.getSenderPeerId()

    await packet.forwardTransform()

    const ack = new Acknowledgement(this.node.paymentChannels, undefined, {
      key: deriveTicketKeyBlinding(packet.header.derivedSecret),
      challenge: packet.oldChallenge
    })

    // Acknowledgement
    setImmediate(async () => this.node.interactions.packet.acknowledgment.interact(sender, await ack.sign(this.node.peerInfo.id)))

    const target = await packet.getTargetPeerId()

    if (this.node.peerInfo.id.isEqual(target)) {
      this.node.output(demo(packet.message.plaintext))
    } else {
      await this.interact(target, packet)
    }

    if (this.queue.length > 0) {
      const index = randomInteger(0, this.queue.length)

      if (index == this.queue.length - 1) {
        return this.handlePacket(this.queue.pop(), token)
      }

      const nextPacket = this.queue[index]

      this.queue[index] = this.queue.pop()

      return this.handlePacket(nextPacket, token)
    }

    this.tokens.push(token)
    return
  }
}

// ==== ONLY FOR TESTING =====
import { decode } from 'rlp'

function demo(plaintext: Uint8Array) {
  const message = decode(Buffer.from(plaintext))

  return `\n\n---------- New Message ----------\nMessage "${message[0].toString()}" latency ${Date.now() -
    Number(message[1].toString())} ms.\n---------------------------------\n\n`
}
// ===========================

export { PacketForwardInteraction }
