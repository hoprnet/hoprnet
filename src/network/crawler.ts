import chalk from 'chalk'
import PeerId from 'peer-id'
import PeerInfo from 'peer-info'
import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import { randomSubset, pubKeyToPeerId, randomInteger } from '../utils'
import { MAX_HOPS, CRAWLING_RESPONSE_NODES } from '../constants'
import Hopr from '..'

const MAX_PARALLEL_REQUESTS = 4

class Crawler<Chain extends HoprCoreConnectorInstance> {
  constructor(public node: Hopr<Chain>) {}

  async crawl(comparator: (peerInfo: PeerInfo) => boolean = () => true): Promise<void> {
    const errors: Error[] = []

    // fast non-inclusion check
    const contactedPeerIds = new Set<string>() // @TODO could be replaced by a bloom filter

    // enumerable
    const unContactedPeerIdArray: PeerId[] = [] // @TODO add new peerIds lazily
    // fast non-inclusion check
    const unContactedPeerIdSet = new Set<string>() // @TODO could be replaced by a bloom filter

    let before = 0 // initialiser
    for (const peerInfo of this.node.peerStore.peers.values()) {
      unContactedPeerIdArray.push(peerInfo.id)

      if (comparator(peerInfo)) {
        before += 1
      }
    }

    /**
     * Get all known nodes that match our requirements.
     */
    const getCurrentNodes = () => {
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
    const getRandomNode = (): PeerId => {
      const index = randomInteger(0, unContactedPeerIdArray.length)

      return unContactedPeerIdArray.splice(index, 1)[0]
    }

    /**
     * Stores the crawling "threads"
     */
    const promises = []

    /**
     * Connect to another peer and returns a promise that resolves to all received nodes
     * that were previously unknown.
     *
     * @param peerId PeerId of the peer that is queried
     */
    const queryNode = async (peerId: PeerId): Promise<void> => {
      let peerIds: PeerId[]

      if (isDone()) {
        return
      }

      // Start additional "threads"
      while (promises.length < MAX_PARALLEL_REQUESTS && unContactedPeerIdArray.length > 0) {
        promises.push(queryNode(getRandomNode()))
      }

      unContactedPeerIdSet.delete(peerId.toB58String())
      contactedPeerIds.add(peerId.toB58String())

      try {
        peerIds = await this.node.interactions.network.crawler.interact(peerId)
      } catch (err) {
        errors.push(err)
      } finally {
        for (let i = 0; i < peerIds.length; i++) {
          if (peerIds[i].isEqual(this.node.peerInfo.id)) {
            continue
          }

          if (!contactedPeerIds.has(peerIds[i].toB58String()) && !unContactedPeerIdSet.has(peerIds[i].toB58String())) {
            unContactedPeerIdSet.add(peerIds[i].toB58String())
            unContactedPeerIdArray.push(peerIds[i])
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

    const addPromises = []

    const addPromiseFactory = (peerId: string) => {
      addPromises.push(PeerInfo.create(PeerId.createFromB58String(peerId)).then((peerInfo: PeerInfo) => this.node.peerStore.put(peerInfo)))
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
  }

  handleCrawlRequest(CrawlResponse: any, Status: any) {
    let self = this
    return (function*() {
      const peers = []

      for (const peerInfo of self.node.peerStore.peers.values()) {
        peers.push(peerInfo)
      }

      const filter = (peerInfo: PeerInfo) => peerInfo.id.pubKey && !peerInfo.id.isEqual(self.node.peerInfo.id)

      const amountOfNodes = Math.min(CRAWLING_RESPONSE_NODES, peers.length)

      const selectedNodes = randomSubset(peers, amountOfNodes, filter).map(peerInfo => peerInfo.id.pubKey.marshal())

      if (selectedNodes.length > 0) {
        yield CrawlResponse.encode({
          status: Status.OK,
          pubKeys: selectedNodes
        })
      } else {
        yield CrawlResponse.encode({
          status: Status.FAIL
        })
      }
    })()
  }
}

export { Crawler }
