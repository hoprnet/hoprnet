import chalk from 'chalk'
import PeerInfo from 'peer-info'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import debug from 'debug'
const log = debug('hopr-core:crawler')

import { getTokens } from '../utils'
import { randomSubset, randomInteger } from '@hoprnet/hopr-utils'
import type { Token } from '../utils'
import { MAX_HOPS, CRAWLING_RESPONSE_NODES } from '../constants'
import type Hopr from '..'

import { CrawlResponse, CrawlStatus } from '../messages'
import PeerId from 'peer-id'

const MAX_PARALLEL_REQUESTS = 4

class Crawler<Chain extends HoprCoreConnector> {
  constructor(public node: Hopr<Chain>) {}

  /**
   *
   * @param comparator
   */
  async crawl(comparator?: (peer: string) => boolean): Promise<void> {
    const errors: Error[] = []

    // fast non-inclusion check
    const contactedPeerIds = new Set<string>() // @TODO could be replaced by a bloom filter

    let unContactedPeers: string[] = [] // @TODO add new peerIds lazily

    let before = 0 // store state before crawling
    let current = 0 // store current state

    log(`Crawling started`)

    unContactedPeers.push(...this.node.network.peerStore.peers.map((entry) => entry.id))

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
    const isDone = (): boolean => contactedPeerIds.size >= MAX_HOPS && current >= MAX_HOPS

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
     *
     * @param peerInfo PeerInfo of the peer that is queried
     */
    const queryNode = async (peer: string, token: Token): Promise<void> => {
      let peerInfos: PeerInfo[]

      if (isDone()) {
        tokens.push(token)
        return
      }

      // Start additional "threads"
      while (tokens.length > 0 && unContactedPeers.length > 0) {
        const token: Token = tokens.pop() as Token
        const currentNode = removeRandomNode()

        if (promises[token] != null) {
          /**
           * @TODO remove this and make sure that the Promise is always
           * already resolved.
           */
          promises[token].then(() => queryNode(currentNode, token))
        } else {
          promises[token] = queryNode(currentNode, token)
        }
      }

      while (true) {
        contactedPeerIds.add(peer)

        try {
          peerInfos = await this.node.interactions.network.crawler.interact(PeerId.createFromB58String(peer))
        } catch (err) {
          errors.push(err)
        } finally {
          if (peerInfos != null && Array.isArray(peerInfos)) {
            for (let i = 0; i < peerInfos.length; i++) {
              const peerString = peerInfos[i].id.toB58String()
              if (peerInfos[i].id.isEqual(this.node.peerInfo.id)) {
                continue
              }

              if (!contactedPeerIds.has(peerString) && !unContactedPeers.includes(peerString)) {
                unContactedPeers.push(peerString)

                if (comparator == null || comparator(peerString)) {
                  current++
                }

                this.node.network.peerStore.push({
                  id: peerString,
                  lastSeen: 0,
                })
                this.node.peerStore.put(peerInfos[i])
              }
            }
          }
        }

        if (unContactedPeers.length > 0 && !isDone()) {
          peer = removeRandomNode()
        } else {
          break
        }
      }

      tokens.push(token)
    }

    if (unContactedPeers.length > 0) {
      let token = tokens.pop()
      promises[token] = queryNode(removeRandomNode(), token)
    }

    if (!isDone()) {
      await Promise.all(promises)
    }

    this.printStatsAndErrors(contactedPeerIds, errors, current, before)

    if (!isDone()) {
      throw Error(`Unable to find enough other nodes in the network.`)
    }
  }

  handleCrawlRequest() {
    return function* (this: Crawler<Chain>) {
      const filter = (entry: { id: string }) => entry.id !== this.node.peerInfo.id.toB58String()

      const amountOfNodes = Math.min(CRAWLING_RESPONSE_NODES, this.node.network.peerStore.peers.length)

      const selectedNodes = randomSubset(this.node.network.peerStore.peers, amountOfNodes, filter)

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
      console.log(
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
