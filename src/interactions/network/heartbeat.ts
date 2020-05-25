import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { AbstractInteraction } from '../abstractInteraction'

import { randomBytes, createHash } from 'crypto'
import { u8aEquals } from '@hoprnet/hopr-utils'

import AbortController from 'abort-controller'
import pipe from 'it-pipe'

import { PROTOCOL_HEARTBEAT } from '../../constants'
import type { Stream, Connection, Handler } from '../../network/transport/types'

import PeerInfo from 'peer-info'
import type PeerId from 'peer-id'

const HASH_FUNCTION = 'blake2s256'

const TWO_SECONDS = 2 * 1000
const HEARTBEAT_TIMEOUT = TWO_SECONDS

class Heartbeat<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_HEARTBEAT]

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: Connection; stream: Stream }) {
    let events = this.node.network.heartbeat
    pipe(
      struct.stream,
      (source: any) => {
        return (async function* () {
          for await (const msg of source) {
            events.emit('beat', struct.connection.remotePeer)
            yield createHash(HASH_FUNCTION).update(msg.slice()).digest()
          }
        })()
      },
      struct.stream
    )
  }

  async interact(counterparty: PeerInfo | PeerId): Promise<void> {
    let struct: Handler

    const abort = new AbortController()
    const signal = abort.signal

    const timeout = setTimeout(() => {
      abort.abort()
    }, HEARTBEAT_TIMEOUT)

    struct = await this.node.dialProtocol(counterparty, this.protocols[0], { signal }).catch(async (err: Error) => {
      const peerInfo = await this.node.peerRouting.findPeer(PeerInfo.isPeerInfo(counterparty) ? counterparty.id : counterparty)

      try {
        let result = await this.node.dialProtocol(peerInfo, this.protocols[0], { signal })
        clearTimeout(timeout)
        return result
      } catch (err) {
        clearTimeout(timeout)
        throw err
      }
    })

    const challenge = randomBytes(16)
    const expectedResponse = createHash(HASH_FUNCTION).update(challenge).digest()

    await pipe(
      /** prettier-ignore */
      [challenge],
      struct.stream,
      async (source: AsyncIterable<Uint8Array>) => {
        let done = false
        for await (const msg of source) {
          if (done == true) {
            continue
          }

          if (u8aEquals(msg, expectedResponse)) {
            done = true
          }
        }
      }
    )
  }
}

export { Heartbeat }
