import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { AbstractInteraction } from '../abstractInteraction'
import { randomBytes, createHash } from 'crypto'
import { u8aEquals, durations } from '@hoprnet/hopr-utils'
import debug from 'debug'
import AbortController from 'abort-controller'
import pipe from 'it-pipe'
import { PROTOCOL_HEARTBEAT } from '../../constants'
import type { Stream, Connection, Handler } from '../../@types/transport'
import type PeerId from 'peer-id'

const error = debug('hopr-core:heartbeat:error')
const verbose = debug('hopr-core:verbose:heartbeat')
const HASH_FUNCTION = 'blake2s256'

export const HEARTBEAT_TIMEOUT = durations.seconds(3)

class Heartbeat<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_HEARTBEAT]

  constructor(
    public node: Hopr<Chain>,
    private options?: {
      timeoutIntentionally?: boolean
    }
  ) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: Connection; stream: Stream }) {
    pipe(
      struct.stream,
      (source: any) => {
        return async function* (this: Heartbeat<Chain>) {
          if (this.options?.timeoutIntentionally) {
            return await new Promise((resolve) => setTimeout(resolve, HEARTBEAT_TIMEOUT + 100))
          }

          for await (const msg of source) {
            this.node.network.heartbeat.emit('beat', struct.connection.remotePeer)
            verbose('beat')
            yield createHash(HASH_FUNCTION).update(msg.slice()).digest()
          }
        }.call(this)
      },
      struct.stream
    )
  }

  async interact(counterparty: PeerId): Promise<void> {
    return new Promise<void>(async (resolve, reject) => {
      // There is an assumption here that we 'know' how to contact this peer
      // and therefore we are immediately trying to dial, rather than checking
      // our peerRouting info first.
      //
      // NB. This is a false assumption for 'ping' and we therefore trigger
      // errors.
      let struct: Handler
      let aborted = false

      const abort = new AbortController()

      const timeout = setTimeout(() => {
        aborted = true
        abort.abort()
        verbose(`heartbeat timeout while querying ${counterparty.toB58String()}`)
        reject(Error(`Timeout while querying ${counterparty.toB58String()}.`))
      }, HEARTBEAT_TIMEOUT)

      try {
        struct = await this.node
          .dialProtocol(counterparty, this.protocols[0], { signal: abort.signal })
          .catch(async (err: Error) => {
            verbose(`heartbeat connection error ${err.name} while dialing ${counterparty.toB58String()} (initial)`)
            const peerInfo = await this.node.peerRouting.findPeer(counterparty)
            //verbose('trying with peer info', peerInfo)
            return await this.node.dialProtocol(peerInfo, this.protocols[0], { signal: abort.signal })
          })
      } catch (err) {
        verbose(`heartbeat connection error ${err.name} while dialing ${counterparty.toB58String()} (subsequent)`)
        clearTimeout(timeout)
        error(err)
        return reject()
      }

      if (aborted) {
        return
      }

      const challenge = randomBytes(16)
      const expectedResponse = createHash(HASH_FUNCTION).update(challenge).digest()

      await pipe([challenge], struct.stream, async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          if (u8aEquals(msg, expectedResponse)) {
            break
          }
        }
      })

      clearTimeout(timeout)

      if (!aborted) {
        resolve()
      }
    })
  }
}

export { Heartbeat }
