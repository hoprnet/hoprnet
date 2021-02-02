/// <reference path="../../@types/libp2p.ts" />

import type { AbstractInteraction } from '../abstractInteraction'
import { randomBytes, createHash } from 'crypto'
import { u8aEquals } from '@hoprnet/hopr-utils'
import debug from 'debug'
import pipe from 'it-pipe'
import { PROTOCOL_HEARTBEAT, HEARTBEAT_TIMEOUT } from '../../constants'
import type { Handler } from 'libp2p'
import type PeerId from 'peer-id'
import { LibP2P } from '../../'
import { dialHelper } from '@hoprnet/hopr-utils'

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

  handler(struct: Handler) {
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

    const struct = await dialHelper(this.node, counterparty, this.protocols, { timeout: HEARTBEAT_TIMEOUT })

    if (struct == undefined) {
      verbose(`Connection to ${counterparty.toB58String()} failed`)
      return -1
    }

    const challenge = randomBytes(16)
    const expectedResponse = createHash(HASH_FUNCTION).update(challenge).digest()

    const response = await pipe(
      // prettier-ignore
      [challenge],
      (struct as Handler).stream,
      async (source: AsyncIterable<Uint8Array>): Promise<Uint8Array | void> => {
        for await (const msg of source) {
          return msg
        }
      }
    )

    if (response != null && u8aEquals(expectedResponse, response.slice())) {
      const elapsedTime = Date.now() - start
      return elapsedTime < 0 ? 0 : elapsedTime
    } else {
      verbose(`Invalid response. Got ${JSON.stringify(response)}`)
      return -1
    }
  }
}

export { Heartbeat }
