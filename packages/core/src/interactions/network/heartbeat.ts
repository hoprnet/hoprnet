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
import Multiaddr from 'multiaddr'

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

  async interact(counterparty: PeerId): Promise<number> {
    const start = Date.now()

    return new Promise<number>(async (resolve, reject) => {
      // There is an assumption here that we 'know' how to contact this peer
      // and therefore we are immediately trying to dial, rather than checking
      // our peerRouting info first.
      //
      // NB. This is a false assumption for 'ping' and we therefore trigger
      // errors.
      let struct: Handler

      const abort = new AbortController()

      const timeout = setTimeout(() => {
        abort.abort()
        verbose(`heartbeat timeout while querying ${counterparty.toB58String()}`)
        reject(Error(`Timeout while querying ${counterparty.toB58String()}.`))
      }, HEARTBEAT_TIMEOUT)

      console.log(`before dialProtocol`)

      console.log(`previous connection`, this.node.connectionManager.connections.get(counterparty.toB58String()))

      try {
        struct = await this.node.dialProtocol(Multiaddr(`/p2p/${counterparty.toB58String()}`), this.protocols[0], {
          signal: abort.signal
        })
      } catch (err) {
        if (err.type === 'aborted') {
          return reject()
        }
        error(`heartbeat connection error ${err.name} while dialing ${counterparty.toB58String()} (initial)`, err)
      }

      console.log(`struct`, struct)

      if (abort.signal.aborted) {
        return reject()
      }

      if (struct == null) {
        const { id, multiaddrs } = await this.node.peerRouting.findPeer(counterparty)

        try {
          struct = await this.node.dialProtocol(id, this.protocols[0], { signal: abort.signal })
        } catch (err) {
          if (err.type === 'aborted') {
            return reject()
          }
          error(`heartbeat connection error ${err.name} while dialing ${counterparty.toB58String()} (subsequent)`)
        }

        console.log(`struct after findPeer`, struct, multiaddrs)
      }

      if (struct == null) {
        return reject()
      }

      const challenge = randomBytes(16)
      const expectedResponse = createHash(HASH_FUNCTION).update(challenge).digest()

      const response = await pipe(
        // prettier-ignore
        [challenge],
        struct.stream,
        async (source: AsyncIterable<Uint8Array>): Promise<Uint8Array | void> => {
          for await (const msg of source) {
            return msg
          }
        }
      )

      clearTimeout(timeout)

      if (response != null && u8aEquals(expectedResponse, response.slice())) {
        resolve(Date.now() - start)
      }

      reject()
    })
  }
}

export { Heartbeat }
