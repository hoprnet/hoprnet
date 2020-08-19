import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import type { Handler } from '../../network/transport/types'

import debug from 'debug'
const log = debug('hopr-core:crawler')

import pipe from 'it-pipe'
import chalk from 'chalk'

import type { AbstractInteraction } from '../abstractInteraction'

import { PROTOCOL_CRAWLING } from '../../constants'
import type PeerInfo from 'peer-info'
import PeerId from 'peer-id'

import { CrawlResponse, CrawlStatus } from '../../messages'

class Crawler<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_CRAWLING]

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: Handler) {
    pipe(
      /* prettier-ignore */
      this.node.network.crawler.handleCrawlRequest(struct.connection),
      struct.stream
    )
  }

  interact(counterparty: PeerId, options: { signal: AbortSignal }): Promise<PeerInfo[]> {
    return new Promise<PeerInfo[]>(async (resolve) => {
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
            const peerInfo = await this.node.peerRouting.findPeer(counterparty)

            return await this.node.dialProtocol(peerInfo, this.protocols[0], { signal: options.signal })
          })
      } catch (err) {
        log(`Could not ask node ${counterparty.toB58String()} for other nodes. Error was: ${chalk.red(err.message)}.`)

        if (!resolved) {
          return resolve([])
        }
        return
      }

      const peerInfos = []
      for await (const encodedResponse of struct.stream.source) {
        let decodedResponse: any
        try {
          decodedResponse = new CrawlResponse(encodedResponse.slice())
        } catch {
          continue
        }

        if (decodedResponse.status !== CrawlStatus.OK) {
          continue
        }

        peerInfos.push(...(await decodedResponse.peerInfos))
      }

      if (!resolved) {
        return resolve(peerInfos)
      }
    })
  }
}

export { Crawler }
