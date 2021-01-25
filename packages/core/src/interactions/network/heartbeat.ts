import type { AbstractInteraction } from '../abstractInteraction'
import { randomBytes, createHash } from 'crypto'
import { u8aEquals } from '@hoprnet/hopr-utils'
import debug from 'debug'
import AbortController from 'abort-controller'
import pipe from 'it-pipe'
import { PROTOCOL_HEARTBEAT, HEARTBEAT_TIMEOUT } from '../../constants'
import type { Stream, Connection, Handler } from 'libp2p'
import type PeerId from 'peer-id'
import { LibP2P } from '../../'

const error = debug('hopr-core:heartbeat:error')
const verbose = debug('hopr-core:verbose:heartbeat')
const HASH_FUNCTION = 'blake2s256'

class Heartbeat implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_HEARTBEAT]

  constructor(
    private node: LibP2P,
    private heartbeat: (remotePeer: PeerId) => void,
    private options?: {
      timeoutIntentionally?: boolean
    }
  ) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { connection: Connection; stream: Stream }) {
    const self = this
    pipe(
      struct.stream,
      (source: any) => {
        return (async function* () {
          if (self.options?.timeoutIntentionally) {
            return await new Promise((resolve) => setTimeout(resolve, HEARTBEAT_TIMEOUT + 100))
          }

          for await (const msg of source) {
            self.heartbeat(struct.connection.remotePeer)
            verbose('beat')
            yield createHash(HASH_FUNCTION).update(msg.slice()).digest()
          }
        })()
      },
      struct.stream
    )
  }

  async interact(counterparty: PeerId): Promise<number | void> {
    const start = Date.now()

    return new Promise<number>(async (resolve, reject) => {
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
        struct = await this.node.dialProtocol(counterparty, this.protocols[0], { signal: abort.signal })
      } catch (err) {
        if (err.type === 'aborted') {
          return
        }
        verbose(`heartbeat connection error ${err.name} while dialing ${counterparty.toB58String()} (initial)`)
      }

      if (abort.signal.aborted) {
        return
      }

      if (struct == null) {
        const { id } = await this.node.peerRouting.findPeer(counterparty)

        try {
          struct = await this.node.dialProtocol(id, this.protocols[0], { signal: abort.signal })
        } catch (err) {
          if (err.type === 'aborted') {
            return
          }
          verbose(`heartbeat connection error ${err.name} while dialing ${counterparty.toB58String()} (subsequent)`)
        }
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
        resolve(Date.now() - start)
      }
    })
  }
}

export { Heartbeat }
