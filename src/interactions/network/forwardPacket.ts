import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { AbstractInteraction } from '../abstractInteraction'

import { getTokens, Token } from '../../utils'
import { ForwardPacket } from '../../messages/forward'

import AbortController from 'abort-controller'
import pipe from 'it-pipe'
import pushable from 'it-pushable'
import type { Pushable } from 'it-pushable'

import { PROTOCOL_FORWARD } from '../../constants'
import { Handler } from '../../network/transport/types'

import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

const MAX_PARALLEL_JOBS = 20

const TWO_SECONDS = 2 * 1000
const FORWARD_TIMEOUT = TWO_SECONDS

type Sender = string

class ForwardPacketInteraction<Chain extends HoprCoreConnector>
  implements AbstractInteraction<Chain> {
  public protocols: string[] = [PROTOCOL_FORWARD]

  private tokens: Token[] = getTokens(MAX_PARALLEL_JOBS)
  private queue: ForwardPacket[] = []
  private promises: Promise<void>[] = []

  private connectionEnds = new Map<Sender, Pushable<Uint8Array>>()

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: any; stream: any }) {
    let forwardPacket: ForwardPacket
    pipe(
      /* prettier-ignore */
      struct.stream,
      async (source: any) => {
        for await (const msg of source) {
          const arr = msg.slice()
          forwardPacket = new ForwardPacket({
            bytes: arr.buffer,
            offset: arr.byteOffset,
          })

          this.queue.push(forwardPacket)

          if (this.tokens.length > 0) {
            const token = this.tokens.pop() as Token
            if (this.promises[token] != null) {
              /**
               * @TODO remove this and make sure that the Promise is always
               * already resolved.
               */
              await this.promises[token]

              this.promises[token] = this.handleForwardPacket(token)
            } else {
              this.handleForwardPacket(token)
            }
          }
        }
      }
    )
  }

  async handleForwardPacket(token: number) {
    let struct: Handler

    let destination: PeerId
    let sender: PeerId
    let forwardPacket: ForwardPacket

    let abort: AbortController
    let signal: AbortSignal

    let timeout: any

    while (this.queue.length > 0) {
      forwardPacket = this.queue.pop() as ForwardPacket

      destination = await PeerId.createFromPubKey(Buffer.from(forwardPacket.destination))

      if (this.node.peerInfo.id.isEqual(destination)) {
        sender = await PeerId.createFromPubKey(Buffer.from(forwardPacket.sender))

        let connectionEnd = this.connectionEnds.get(sender.toB58String())
        if (connectionEnd != null) {
          connectionEnd.push(forwardPacket.payload)
        } else {
          throw Error(`Received unexpected forwarded packet.`)
        }

        continue
      }

      abort = new AbortController()
      signal = abort.signal

      timeout = setTimeout(() => {
        // @TODO add short-term storage here
        abort.abort()
      }, FORWARD_TIMEOUT)

      struct = await this.node
        .dialProtocol(destination, this.protocols[0], { signal })
        .catch(async (err: Error) => {
          const peerInfo = await this.node.peerRouting.findPeer(destination)

          try {
            let result = await this.node.dialProtocol(peerInfo, this.protocols[0], { signal })
            clearTimeout(timeout)
            return result
          } catch (err) {
            clearTimeout(timeout)
            throw err
          }
        })

      await pipe(
        /* prettier-ignore */
        [forwardPacket],
        struct.stream
      )
    }

    this.tokens.push(token)
  }

  async interact(counterparty: PeerInfo | PeerId, relay: PeerId | PeerInfo): Promise<any> {
    let struct: Handler

    let relayPeerId = PeerInfo.isPeerInfo(relay) ? relay.id : relay
    let counterpartyPeerId = PeerInfo.isPeerInfo(counterparty) ? counterparty.id : counterparty

    const abort = new AbortController()
    const signal = abort.signal

    const timeout = setTimeout(() => {
      abort.abort()
    }, FORWARD_TIMEOUT)

    struct = await this.node
      .dialProtocol(relay, this.protocols[0], { signal })
      .catch(async (err: Error) => {
        const peerInfo = await this.node.peerRouting.findPeer(relayPeerId)

        try {
          let result = await this.node.dialProtocol(peerInfo, this.protocols[0], { signal })
          clearTimeout(timeout)
          return result
        } catch (err) {
          clearTimeout(timeout)
          throw err
        }
      })

    const connectionEnd = pushable<Uint8Array>()
    this.connectionEnds.set(counterpartyPeerId.toB58String(), connectionEnd)

    let self = this
    return {
      source: connectionEnd,
      sink: async function (source: any) {
        pipe(
          /* prettier-ignore */
          source,
          (source: any) => {
            return (async function* () {
              for await (let msg of source) {
                yield new ForwardPacket(undefined, {
                  destination: counterpartyPeerId,
                  sender: self.node.peerInfo.id,
                  payload: msg.slice(),
                })
              }
            })()
          },
          struct.stream
        )
      },
    }
  }
}

export { ForwardPacketInteraction }
