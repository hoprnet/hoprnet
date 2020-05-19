import type Hopr from '../../'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import pipe from 'it-pipe'
import chalk from 'chalk'

import type { AbstractInteraction, Duplex } from '../abstractInteraction'

import { PROTOCOL_CRAWLING } from '../../constants'
import type PeerInfo from 'peer-info'
import AbortController from 'abort-controller'

import { CrawlResponse, CrawlStatus } from '../../messages'

const CRAWL_TIMEOUT = 1 * 1000

class Crawler<Chain extends HoprCoreConnector> implements AbstractInteraction<Chain> {
  protocols: string[] = [PROTOCOL_CRAWLING]

  constructor(public node: Hopr<Chain>) {
    this.node.handle(this.protocols, this.handler.bind(this))
  }

  handler(struct: { stream: any }) {
    pipe(
      /* prettier-ignore */
      this.node.network.crawler.handleCrawlRequest(),
      struct.stream
    )
  }

  async interact(counterparty: PeerInfo): Promise<PeerInfo[]> {
    let struct: {
      stream: Duplex
      protocol: string
    }

    const abort = new AbortController()
    const signal = abort.signal

    const timeout = setTimeout(abort.abort.bind(abort), CRAWL_TIMEOUT)

    try {
      struct = await this.node.dialProtocol(counterparty, this.protocols[0], { signal }).catch(async (_: Error) => {
        const peerInfo = await this.node.peerRouting.findPeer(counterparty.id)

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
      this.node.log(`Could not ask node ${counterparty.id.toB58String()} for other nodes. Error was: ${chalk.red(err.message)}.`)
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
