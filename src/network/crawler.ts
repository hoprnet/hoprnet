import chalk from 'chalk'
import PeerId from 'peer-id'
import PeerInfo from 'peer-info'

import Queue = require('promise-queue')

import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import { randomSubset, pubKeyToPeerId } from '../utils'
import { MAX_HOPS, PROTOCOL_CRAWLING, CRAWLING_RESPONSE_NODES } from '../constants'
import Hopr from '..'

const MAX_PARALLEL_REQUESTS = 4
const QUEUE_MAX_SIZE = Infinity

class Crawler<Chain extends HoprCoreConnectorInstance> {
  constructor(public node: Hopr<Chain>) {}

  async crawl(comparator: (peerInfo: PeerInfo) => boolean = () => true) {
    const queue = new Queue(MAX_PARALLEL_REQUESTS, QUEUE_MAX_SIZE)
    let finished = false,
      first = true

    const errors = []

    /**
     * Connect to another peer and returns a promise that resolves to all received nodes
     * that were previously unknown.
     *
     * @param peerId PeerId of the peer that is queried
     */
    async function queryNode(peerId: PeerId) {
      let peerIds = await this.node.interactions.network.crawler.interact(peerId)

      const set = new Set<string>()
      peerIds = peerIds.reduce((acc: PeerId[], peerId: PeerId) => {
        if (peerId.isEqual(this.node.peerInfo.id)) {
          return acc
        }

        if (!set.has(peerId.toB58String()) && !this.node.peerBook.has(peerId.toB58String())) {
          acc.push(peerId)
          this.node.peerBook.put(new PeerInfo(peerId))
          set.add(peerId.toB58String())
        }
      }, [])
    }

    /**
     * Decides whether we have enough peers to build a path and initiates some queries
     * if that's not the case.
     *
     * @param peerIds array of peerIds
     */
    function processResults(peerIds: PeerId[]) {
      const now = this.node.peerBook.getAllArray().filter(comparator).length

      if (finished) {
        return
      }

      if (!first && now >= MAX_HOPS) {
        if (errors.length > 0) {
          this.node.log(
            `Errors while crawling:${errors.reduce((acc, err) => {
              acc += `\n${chalk.red(err.message)}`
              return acc
            }, '')}`
          )
        }

        finished = true

        this.node.log(`Received ${now - before} new node${now - before == 1 ? '' : 's'}.`)
        this.node.log(`Now holding peer information of ${now} node${now == 1 ? '' : 's'} in the network.`)

        return
      }

      first = false

      if (peerIds.length > 0) {
        const subset = randomSubset(peerIds, Math.min(peerIds.length, MAX_PARALLEL_REQUESTS))

        subset.forEach((peerId: PeerId) => {
          queue
            .add(() => queryNode(peerId))
            .then((peerIds: PeerId[]) => {
              processResults(peerIds)
            })
            .catch((err: Error) => {
              errors.push(err)
              return processResults([])
            })
        })
      }

      if (queue.getPendingLength() == 0 && queue.getQueueLength() == 0) {
        if (errors.length > 0) {
          this.node.log(`Errors while crawling:${errors.reduce((acc, err) => `\n${chalk.red(err.message)}`, '')}`)
        }

        this.node.log(`Received ${now - before} new node${now - before == 1 ? '' : 's'}.`)
        this.node.log(`Now holding peer information of ${now} node${now == 1 ? '' : 's'} in the network.`)

        throw Error('Unable to find enough other nodes in the network.')
      }
    }

    const before = this.node.peerBook.getAllArray().filter(comparator).length

    let nodes = this.node.peerBook.getAllArray().map(peerInfo => peerInfo.id)

    processResults(nodes)
  }

  handleCrawlRequest<Chain extends HoprCoreConnectorInstance>(CrawlResponse: any, Status: any) {
    return (function*() {
      const peers = this.node.peerBook.getAllArray()
      const filter = (peerInfo: PeerInfo) => peerInfo.id.pubKey && !peerInfo.id.isEqual(this.node.peerInfo.id)

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
