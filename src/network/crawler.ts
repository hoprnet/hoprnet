import chalk from 'chalk'
import PeerId from 'peer-id'
import PeerInfo from 'peer-info'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import { randomSubset, randomInteger } from '../utils'
import { MAX_HOPS, CRAWLING_RESPONSE_NODES } from '../constants'
import Hopr from '..'

import { CrawlResponse, CrawlStatus } from '../messages'

const MAX_PARALLEL_REQUESTS = 4

class Crawler<Chain extends HoprCoreConnectorInstance> {
  constructor(public node: Hopr<Chain>) {}

  async crawl(comparator: (peerInfo: PeerInfo) => boolean = () => true): Promise<void> {
    const errors: Error[] = []

    // fast non-inclusion check
    const contactedPeerIds = new Set<string>() // @TODO could be replaced by a bloom filter

    // enumerable
    const unContactedPeerIdArray: PeerInfo[] = [] // @TODO add new peerIds lazily
    // fast non-inclusion check
    const unContactedPeerIdSet = new Set<string>() // @TODO could be replaced by a bloom filter

    let before = 0 // store current state
    for (const peerInfo of this.node.peerStore.peers.values()) {
      unContactedPeerIdArray.push(peerInfo)

      if (comparator(peerInfo)) {
        before += 1
      }
    }

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
      const index = randomInteger(0, unContactedPeerIdArray.length)

      if (index == unContactedPeerIdArray.length - 1) {
        return unContactedPeerIdArray.pop()
      }

      const selected: PeerInfo = unContactedPeerIdArray[index]
      unContactedPeerIdArray[index] = unContactedPeerIdArray.pop()

      return selected
    }

    /**
     * Stores the crawling "threads"
     */
    const promises = []

    /**
     * Connect to another peer and returns a promise that resolves to all received nodes
     * that were previously unknown.
     *
     * @param peerInfo PeerInfo of the peer that is queried
     */
    const queryNode = async (peerInfo: PeerInfo): Promise<void> => {
      let peerInfos: PeerInfo[]

      if (isDone()) {
        return
      }

      // Start additional "threads"
      while (promises.length < MAX_PARALLEL_REQUESTS && unContactedPeerIdArray.length > 0) {
        promises.push(queryNode(getRandomNode()))
      }

      unContactedPeerIdSet.delete(peerInfo.id.toB58String())
      contactedPeerIds.add(peerInfo.id.toB58String())

      try {
        peerInfos = await this.node.interactions.network.crawler.interact(peerInfo)
      } catch (err) {
        errors.push(err)
      } finally {
        for (let i = 0; i < peerInfos.length; i++) {
          if (peerInfos[i].id.isEqual(this.node.peerInfo.id)) {
            continue
          }

          if (!contactedPeerIds.has(peerInfos[i].id.toB58String()) && !unContactedPeerIdSet.has(peerInfos[i].id.toB58String())) {
            unContactedPeerIdSet.add(peerInfos[i].id.toB58String())
            unContactedPeerIdArray.push(peerInfos[i])
            this.node.peerStore.put(peerInfos[i])
          }
        }
      }

      if (unContactedPeerIdArray.length > 0) {
        return queryNode(getRandomNode())
      } else {
        return
      }
    }

    for (let i = 0; i < MAX_PARALLEL_REQUESTS; i++) {
      if (unContactedPeerIdArray.length > 0) {
        promises.push(queryNode(getRandomNode()))
      }
    }

    if (!isDone()) {
      await Promise.all(promises)
    }

    const addPromises: Promise<void>[] = []

    const addPromiseFactory = (peerIdString: string) => {
      const peerId = PeerId.createFromB58String(peerIdString)

      if (!this.node.peerStore.has(peerId)) {
        addPromises.push(
          PeerInfo.create(peerId).then((peerInfo: PeerInfo) => {
            this.node.peerStore.put(peerInfo)
          })
        )
      } else {
        return Promise.resolve()
      }
    }

    unContactedPeerIdSet.forEach(addPromiseFactory)
    contactedPeerIds.forEach(addPromiseFactory)

    await Promise.all(addPromises)

    if (errors.length > 0) {
      this.node.log(
        `Errors while crawling:${errors.reduce((acc, err) => {
          acc += `\n\t${chalk.red(err.message)}`
          return acc
        }, '')}`
      )
    }

    const now = getCurrentNodes()
    let contactedNodes = ``
    contactedPeerIds.forEach((peerId: string) => {
      contactedNodes += `\n        ${peerId}`
    })
    this.node.log(
      `Crawling results:\n    ${chalk.yellow(`contacted nodes:`)}: ${contactedNodes}\n    ${chalk.green(`new nodes`)}: ${now - before} node${
        now - before == 1 ? '' : 's'
      }\n    total: ${now} node${now == 1 ? '' : 's'}`
    )

    if (!isDone()) {
      throw Error(`Unable to find enough other nodes in the network.`)
    }

    unContactedPeerIdSet.clear()
    contactedPeerIds.clear()
  }

  handleCrawlRequest() {
    let self = this
    return (function*() {
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
          peerInfos: selectedNodes
        })
      } else {
        yield new CrawlResponse(undefined, {
          status: CrawlStatus.FAIL
        })
      }
    })()
  }
}

export { Crawler }
