import chalk from 'chalk'
import PeerInfo from 'peer-info'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import AbortController from 'abort-controller'

import debug from 'debug'
const log = debug('hopr-core:crawler')

import { getTokens } from '../utils'
import { randomSubset, randomInteger } from '@hoprnet/hopr-utils'
import type { Token } from '../utils'
import { MAX_HOPS, CRAWLING_RESPONSE_NODES } from '../constants'
import type Hopr from '..'

import { CrawlResponse, CrawlStatus } from '../messages'
import PeerId from 'peer-id'
import type { Connection } from './transport/types'
import type { Entry } from './peerStore'

const MAX_PARALLEL_REQUESTS = 7
export const CRAWL_TIMEOUT = 1 * 1000

class Crawler<Chain extends HoprCoreConnector> {
  constructor(
    public node: Hopr<Chain>,
    private options?: {
      timeoutIntentionally?: boolean
    }
  ) {}

  /**
   *
   * @param comparator
   */
  async crawl(comparator?: (peer: string) => boolean): Promise<void> {
    return new Promise(async (resolve) => {
      let aborted = false

      const errors: Error[] = []

      // fast non-inclusion check
      const contactedPeerIds = new Set<string>() // @TODO could be replaced by a bloom filter

      let unContactedPeers: string[] = [] // @TODO add new peerIds lazily

      let before = 0 // store state before crawling
      let current = 0 // store current state

      const abort = new AbortController()

      const timeout = setTimeout(() => {
        aborted = true

        abort.abort()

        this.printStatsAndErrors(contactedPeerIds, errors, current, before)

        resolve()
      }, CRAWL_TIMEOUT)

      log(`Crawling started`)

      this.node.network.peerStore.cleanupBlacklist()

      unContactedPeers.push(...this.node.network.peerStore.peers.map((entry: Entry) => entry.id))

      if (comparator != null) {
        for (let i = 0; i < unContactedPeers.length; i++) {
          if (comparator(unContactedPeers[i])) {
            before += 1
          }
        }
      } else {
        before = unContactedPeers.length
      }

      current = before

      const tokens: Token[] = getTokens(MAX_PARALLEL_REQUESTS)

      /**
       * Check if we're finished
       */
      const isDone = (): boolean => aborted || (contactedPeerIds.size >= MAX_HOPS && current >= MAX_HOPS)

      /**
       * Returns a random node and removes it from the array.
       */
      const removeRandomNode = (): string => {
        if (unContactedPeers.length == 0) {
          throw Error(`Cannot pick a random node because there are none.`)
        }

        const index = randomInteger(0, unContactedPeers.length)

        if (index == unContactedPeers.length - 1) {
          return unContactedPeers.pop() as string
        }

        const selected: string = unContactedPeers[index]
        unContactedPeers[index] = unContactedPeers.pop() as string

        return selected
      }

      /**
       * Stores the crawling "threads"
       */
      const promises: Promise<void>[] = Array.from({ length: MAX_PARALLEL_REQUESTS })

      /**
       * Connect to another peer and returns a promise that resolves to all received nodes
       * that were previously unknown.
       */
      const queryNode = async (peer: string, token: Token): Promise<void> => {
        let peerInfos: PeerInfo[]

        if (isDone()) {
          promises[token] = undefined
          tokens.push(token)
          return
        }

        // Start additional "threads"
        while (tokens.length > 0 && unContactedPeers.length > 0) {
          const token: Token = tokens.pop() as Token

          promises[token] = queryNode(removeRandomNode(), token)
        }

        while (true) {
          contactedPeerIds.add(peer)

          try {
            log(`querying ${chalk.blue(peer)}`)
            peerInfos = await this.node.interactions.network.crawler.interact(PeerId.createFromB58String(peer), {
              signal: abort.signal,
            })
            log(
              `received [${peerInfos.map((p) => chalk.blue(p.id.toB58String())).join(', ')}] from peer ${chalk.blue(
                peer
              )}`
            )
          } catch (err) {
            peerInfos = undefined
            errors.push(err)
            continue
          }

          for (let i = 0; i < peerInfos.length; i++) {
            const peerString = peerInfos[i].id.toB58String()

            if (peerInfos[i].id.isEqual(this.node.peerInfo.id)) {
              continue
            }

            if (!contactedPeerIds.has(peerString) && !unContactedPeers.includes(peerString)) {
              unContactedPeers.push(peerString)

              let beforeInserting = this.node.network.peerStore.length
              this.node.network.peerStore.push({
                id: peerString,
                lastSeen: 0,
              })

              if (comparator == null || comparator(peerString)) {
                current = current + this.node.network.peerStore.length - beforeInserting
              }
              this.node.peerStore.put(peerInfos[i])
            }
          }

          if (unContactedPeers.length == 0 || isDone()) {
            break
          }

          peer = removeRandomNode()
        }

        promises[token] = undefined
        tokens.push(token)
      }

      if (unContactedPeers.length > 0) {
        let token = tokens.pop()
        promises[token] = queryNode(removeRandomNode(), token)
      }

      if (!isDone()) {
        await Promise.all(promises)
      }

      if (!aborted) {
        clearTimeout(timeout)

        this.printStatsAndErrors(contactedPeerIds, errors, current, before)

        resolve()
      }

      // @TODO re-enable this once routing is done properly.
      // if (!isDone()) {
      //   throw Error(`Unable to find enough other nodes in the network.`)
      // }
    })
  }

  handleCrawlRequest(conn?: Connection) {
    return async function* (this: Crawler<Chain>) {
      const amountOfNodes = Math.min(CRAWLING_RESPONSE_NODES, this.node.network.peerStore.peers.length)

      const selectedNodes = randomSubset(
        this.node.network.peerStore.peers,
        amountOfNodes,
        (entry: Entry) =>
          entry.id !== this.node.peerInfo.id.toB58String() &&
          (conn == null || entry.id !== conn.remotePeer.toB58String())
      )

      if (this.options?.timeoutIntentionally) {
        await new Promise((resolve) => setTimeout(resolve, CRAWL_TIMEOUT + 100))
      }

      if (selectedNodes.length > 0) {
        yield new CrawlResponse(undefined, {
          status: CrawlStatus.OK,
          peerInfos: selectedNodes.map((peer) => {
            const result = new PeerInfo(PeerId.createFromB58String(peer.id))

            const found = this.node.peerStore.get(PeerId.createFromB58String(peer.id))

            if (found) {
              found.multiaddrs.forEach((ma) => result.multiaddrs.add(ma))
            }

            return result
          }),
        })
      } else {
        yield new CrawlResponse(undefined, {
          status: CrawlStatus.FAIL,
        })
      }
    }.call(this)
  }

  printStatsAndErrors(contactedPeerIds: Set<string>, errors: Error[], now: number, before: number) {
    if (errors.length > 0) {
      log(
        `Errors while crawling:${errors.reduce((acc, err) => {
          acc += `\n\t${chalk.red(err.message)}`
          return acc
        }, '')}`
      )
    }

    let contactedNodes = ``
    contactedPeerIds.forEach((peerId: string) => {
      contactedNodes += `\n        ${peerId}`
    })

    console.log(
      `Crawling results:\n    ${chalk.yellow(`contacted nodes:`)}: ${contactedNodes}\n    ${chalk.green(
        `new nodes`
      )}: ${now - before} node${now - before == 1 ? '' : 's'}\n    total: ${now} node${now == 1 ? '' : 's'}`
    )
  }
}

export { Crawler }
