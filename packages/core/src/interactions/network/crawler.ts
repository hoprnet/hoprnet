import debug from 'debug'
import pipe from 'it-pipe'
import chalk from 'chalk'
import type { AbstractInteraction } from '../abstractInteraction'
import { PROTOCOL_CRAWLING } from '../../constants'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'
import { CrawlResponse, CrawlStatus } from '../../messages'
import { LibP2P } from '../../'
import type { Connection, Handler } from 'libp2p'

const log = debug('hopr-core:crawler')
const verbose = debug('hopr-core:verbose:crawl-interaction')

class Crawler implements AbstractInteraction {
  protocols: string[] = [PROTOCOL_CRAWLING]

  constructor(private node: LibP2P, private handleCrawlRequest: (conn: Connection) => void) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: Handler) {
    pipe(this.handleCrawlRequest(struct.connection), struct.stream)
  }

  interact(counterparty: PeerId, options: { signal: AbortSignal }): Promise<Multiaddr[]> {
    verbose('crawl interact', counterparty.toB58String())
    return new Promise<Multiaddr[]>(async (resolve) => {
      let resolved = false
      const onAbort = () => {
        options.signal.removeEventListener('abort', onAbort)

        if (!resolved) {
          resolve([])
          resolved = true
        }
      }
      options.signal.addEventListener('abort', () => resolve([]))

      let struct: Handler

      try {
        struct = await this.node
          .dialProtocol(counterparty, this.protocols[0], { signal: options.signal })
          .catch(async (_: Error) => {
            const { id } = await this.node.peerRouting.findPeer(counterparty)

            return await this.node.dialProtocol(id, this.protocols[0], { signal: options.signal })
          })
      } catch (err) {
        log(`Could not ask node ${counterparty.toB58String()} for other nodes. Error was: ${chalk.red(err.message)}.`)

        if (!resolved) {
          return resolve([])
        }
        return
      }

      const addresses = []
      for await (const encodedResponse of struct.stream.source) {
        let decodedResponse: any

        let _received = encodedResponse.slice()
        try {
          decodedResponse = new CrawlResponse({
            bytes: _received.buffer,
            offset: _received.byteOffset
          })
        } catch {
          continue
        }

        if (decodedResponse.status !== CrawlStatus.OK) {
          continue
        }

        addresses.push(...(await decodedResponse.addresses))
      }

      if (!resolved) {
        return resolve(addresses)
      }
    })
  }
}

export { Crawler }
