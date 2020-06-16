import chalk from 'chalk'
import PeerInfo from 'peer-info'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import { getTokens } from '../utils'
import { randomSubset, randomInteger } from '@hoprnet/hopr-utils'
import type { Token } from '../utils'
import { MAX_HOPS, CRAWLING_RESPONSE_NODES } from '../constants'
import type Hopr from '..'

import { CrawlResponse, CrawlStatus } from '../messages'

const MAX_PARALLEL_REQUESTS = 4

class Crawler<Chain extends HoprCoreConnector> {
  constructor(public node: Hopr<Chain>) {}

  async crawl(comparator: (peerInfo: PeerInfo) => boolean = () => true): Promise<void> {
    const errors: Error[] = []

    // fast non-inclusion check
    const contactedPeerIds = new Set<string>() // @TODO could be replaced by a bloom filter

    // enumerable
    const unContactedPeerIdArray: PeerInfo[] = [] // @TODO add new peerIds lazily
    // fast non-inclusion check
    const unContactedPeerIdSet = new Set<string>() // @TODO replace this by a sorted array

    let before = 0 // store current state
    for (const peerInfo of this.node.peerStore.peers.values()) {
      unContactedPeerIdArray.push(peerInfo)

      if (comparator(peerInfo)) {
        before += 1
      }
    }

    const tokens: Token[] = getTokens(MAX_PARALLEL_REQUESTS)

    /**
     * Get all known nodes that match our requirements.
     */
    const getCurrentNodes = (): number => {
      let currentNodes = 0

      for (const peerInfo of this.node.peerStore.peers.values()) {
        if (comparator(peerInfo) == true) {
          currentNodes += 1
        }
      }

      return currentNodes
    }

    /**
     * Check if we're finished
     */
    const isDone = (): boolean => {
      return contactedPeerIds.size >= MAX_HOPS && getCurrentNodes() >= MAX_HOPS
    }

    /**
     * Returns a random node and removes it from the array.
     */
    const getRandomNode = (): PeerInfo => {
      if (unContactedPeerIdArray.length == 0) {
        throw Error(`Cannot pick a random node because there are none.`)
      }

      const index = randomInteger(0, unContactedPeerIdArray.length)

      if (index == unContactedPeerIdArray.length - 1) {
        return unContactedPeerIdArray.pop() as PeerInfo
      }

      const selected: PeerInfo = unContactedPeerIdArray[index]
      unContactedPeerIdArray[index] = unContactedPeerIdArray.pop() as PeerInfo

      return selected
    }

    /**
     * Stores the crawling "threads"
     */
    const promises: Promise<void>[] = []

    /**
     * Connect to another peer and returns a promise that resolves to all received nodes
     * that were previously unknown.
     *
     * @param peerInfo PeerInfo of the peer that is queried
     */
    const queryNode = async (peerInfo: PeerInfo, token: Token): Promise<void> => {
      let peerInfos: PeerInfo[]

      if (isDone()) {
        tokens.push(token)
        return
      }

      // Start additional "threads"
      while (tokens.length > 0 && unContactedPeerIdArray.length > 0) {
        const token: Token = tokens.pop() as Token
        const currentNode = getRandomNode()

        if (promises[token] != null) {
          /**
           * @TODO remove this and make sure that the Promise is always
           * already resolved.
           */
          await promises[token]

          promises[token] = queryNode(currentNode, token)
        } else {
          promises.push(queryNode(currentNode, token))
        }
      }

      unContactedPeerIdSet.delete(peerInfo.id.toB58String())
      contactedPeerIds.add(peerInfo.id.toB58String())

      try {
        peerInfos = await this.node.interactions.network.crawler.interact(peerInfo)
      } catch (err) {
        errors.push(err)
      } finally {
        if (peerInfos != null && Array.isArray(peerInfos)) {
          for (let i = 0; i < peerInfos.length; i++) {
            if (peerInfos[i].id.isEqual(this.node.peerInfo.id)) {
              continue
            }

            if (
              !contactedPeerIds.has(peerInfos[i].id.toB58String()) &&
              !unContactedPeerIdSet.has(peerInfos[i].id.toB58String())
            ) {
              unContactedPeerIdSet.add(peerInfos[i].id.toB58String())
              unContactedPeerIdArray.push(peerInfos[i])
              this.node.peerStore.put(peerInfos[i])
            }
          }
        }
      }

      if (unContactedPeerIdArray.length > 0) {
        return queryNode(getRandomNode(), token)
      }

      tokens.push(token)
      return
    }

    for (let i = 0; i < MAX_PARALLEL_REQUESTS && unContactedPeerIdArray.length > 0; i++) {
      promises.push(queryNode(getRandomNode(), tokens.pop() as Token))
    }

    if (!isDone()) {
      await Promise.all(promises)
    }

    this.printStatsAndErrors(contactedPeerIds, errors, getCurrentNodes(), before)

    if (!isDone()) {
      throw Error(`Unable to find enough other nodes in the network.`)
    }

    unContactedPeerIdSet.clear()
    contactedPeerIds.clear()
  }

  handleCrawlRequest() {
    let self = this
    return (function* () {
      const peers = []

      for (const peerInfo of self.node.peerStore.peers.values()) {
        peers.push(peerInfo)
      }

      const filter = (peerInfo: PeerInfo) => peerInfo.id.pubKey && !peerInfo.id.isEqual(self.node.peerInfo.id)

      const amountOfNodes = Math.min(CRAWLING_RESPONSE_NODES, peers.length)

      const selectedNodes = randomSubset(peers, amountOfNodes, filter)

      if (selectedNodes.length > 0) {
        yield new CrawlResponse(undefined, {
          status: CrawlStatus.OK,
          peerInfos: selectedNodes,
        })
      } else {
        yield new CrawlResponse(undefined, {
          status: CrawlStatus.FAIL,
        })
      }
    })()
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
