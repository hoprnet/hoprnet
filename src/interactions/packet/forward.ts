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

import { getTokens, Token, randomInteger, u8aToHex } from '../../utils'

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

    await pipe(
      /* prettier-ignore */
      [packet],
      struct.stream
    )
  }

  async handler(struct: { stream: any }): Promise<void> {
    let packet: Packet<Chain>
    pipe(
      /* pretttier-ignore */
      struct.stream,
      async (source: AsyncIterable<Uint8Array>): Promise<void> => {
        for await (const msg of source) {
          const arr = msg.slice()
          packet = new Packet(this.node, {
            bytes: arr.buffer,
            offset: arr.byteOffset
          })

          if (this.tokens.length > 0) {
            const token = this.tokens.pop()
            if (this.promises[token] != null) {
              /**
               * @TODO remove this and make sure that the Promise is always
               * already resolved.
               */
              await this.promises[token]

              this.promises[token] = this.handlePacket(packet, token)
            } else {
              this.handlePacket(packet, token)
            }
          } else {
            this.queue.push(packet)
          }
        }
      }
    )
  }

  async handlePacket(packet: Packet<Chain>, token: number): Promise<void> {
    await packet.forwardTransform()

    const [sender, target] = await Promise.all([
      /* prettier-ignore */
      packet.getSenderPeerId(),
      packet.getTargetPeerId()
    ])

    const ack = new Acknowledgement(this.node.paymentChannels, undefined, {
      key: deriveTicketKeyBlinding(packet.header.derivedSecret),
      challenge: packet.oldChallenge
    })

    // Acknowledgement

    setImmediate(async () => {
      this.node.interactions.packet.acknowledgment.interact(sender, await ack.sign(this.node.peerInfo.id))
    })

    if (this.node.peerInfo.id.isEqual(target)) {
      packet.message.encrypted = false
      this.node.output(packet.message.plaintext)
    } else {
      await this.interact(target, packet)
    }

    // Check for unserviced packets
    if (this.queue.length > 0) {
      // Pick a random one
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
// import { decode } from 'rlp'

// function demo(plaintext: Uint8Array) {
//   const message = decode(Buffer.from(plaintext))

//   return `\n\n---------- New Message ----------\nMessage "${message[0].toString()}" latency ${Date.now() -
//     Number(message[1].toString())} ms.\n---------------------------------\n\n`
// }
// ===========================

export { PacketForwardInteraction }
