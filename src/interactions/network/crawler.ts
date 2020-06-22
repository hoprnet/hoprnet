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
import AbortController from 'abort-controller'

import { CrawlResponse, CrawlStatus } from '../../messages'

const CRAWL_TIMEOUT = 1 * 1000

class Crawler<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_CRAWLING]

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: Handler) {
    pipe(
      /* prettier-ignore */
      this.node.network.crawler.handleCrawlRequest(),
      struct.stream
    )
  }

  async interact(counterparty: PeerId): Promise<PeerInfo[]> {
    let struct: Handler

    const abort = new AbortController()
    const signal = abort.signal

    const timeout = setTimeout(abort.abort.bind(abort), CRAWL_TIMEOUT)

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0], { signal }).catch(async (_: Error) => {
        const peerInfo = await this.node.peerRouting.findPeer(counterparty)

        try {
          let result = await this.node.dialProtocol(peerInfo, this.protocols[0], { signal })
          clearTimeout(timeout)
          return result
        } catch (err) {
          clearTimeout(timeout)
          throw err
        }
      })
    } catch (err) {
      this.node.log(
        `Could not ask node ${counterparty.toB58String()} for other nodes. Error was: ${chalk.red(err.message)}.`
      )
      return []
    }

    if (signal.aborted) {
      return []
    }

    return await pipe(
      /** prettier-ignore */
      struct.stream,
      collect
    )
  }
}

async function collect(source: AsyncIterable<Uint8Array>) {
  const peerInfos = []
  for await (const encodedResponse of source) {
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

  return peerInfos
}

export { Crawler }
