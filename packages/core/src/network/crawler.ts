import chalk from 'chalk'
import PeerInfo from 'peer-info'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import AbortController from 'abort-controller'
import { getTokens } from '../utils'
import { randomSubset, randomInteger } from '@hoprnet/hopr-utils'
import type { Token } from '../utils'
import { MAX_HOPS, CRAWLING_RESPONSE_NODES } from '../constants'
import type Hopr from '..'
import { CrawlResponse, CrawlStatus } from '../messages'
import PeerId from 'peer-id'
import type { Connection } from './transport/types'
import type { Entry } from './peerStore'
import { peerHasOnlyPublicAddresses, isOnPrivateNet, PRIVATE_NETS } from '../filters'
import debug from 'debug'
import Multiaddr from 'multiaddr'
const log = debug('hopr-core:crawler')
const verbose = debug('hopr-core:verbose:crawler')

const MAX_PARALLEL_REQUESTS = 7
export const CRAWL_TIMEOUT = 2 * 1000

export const shouldIncludePeerInCrawlResponse = (peer: Multiaddr, them: Multiaddr): boolean => {
  // We are being requested a crawl from a node that is on a remote network, so
  // it does not benefit them for us to give them addresses on our private
  // network, therefore let's first filter these out.
  if (!them.nodeAddress().address.match(PRIVATE_NETS) && isOnPrivateNet(peer)) {
    verbose('rejecting peer from crawl results as it only has private addresses, and the requesting node is remote')
    return false // Reject peer
  }
  return true
}

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
    verbose('creating a crawl')
    return new Promise(async (resolve) => {
      let aborted = false

      const errors: Error[] = []

      // fast non-inclusion check
      const contactedPeerIds = new Set<string>() // @TODO could be replaced by a bloom filter

      let unContactedPeers: string[] = [] // @TODO add new peerIds lazily

      let before = 0 // number of peers before crawling
      let current = 0 // current number of peers

      const abort = new AbortController()

      const timeout = setTimeout(() => {
        aborted = true
        verbose('aborting crawl due to timeout')
        abort.abort()
        this.printStatsAndErrors(contactedPeerIds, errors, current, before)
        resolve()
      }, CRAWL_TIMEOUT)

      log(`Crawling started`)

      this.node.network.peerStore.cleanupBlacklist()

      unContactedPeers.push(...this.node.network.peerStore.peers.map((entry: Entry) => entry.id))
      verbose(`added ${unContactedPeers.length} peers to crawl list`)

      if (comparator != null) {
        for (let i = 0; i < unContactedPeers.length; i++) {
          if (comparator(unContactedPeers[i])) {
            before += 1
          }
        }
        verbose('comparator passed; number of peers before: ', before)
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
       * Swaps in the last element of the array with the element removed.
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
            const peerId = PeerId.createFromB58String(peer)
            peerInfos = await this.node.interactions.network.crawler.interact(peerId, {
              signal: abort.signal,
            })

            const peerInfo = this.node.peerStore.get(peerId)
            if (peerInfo && peerHasOnlyPublicAddresses(peerInfo)) {
              // The node we are connecting to is on a remote network
              // and gives us addresses on a private network, then they are
              // not going to work for us. We should filter these out when we are
              // requested for a crawl, but in this instance they have given us
              // some anyway.
              peerInfos.forEach((p) => {
                p.multiaddrs.forEach((ma) => {
                  if (isOnPrivateNet(ma)) {
                    p.multiaddrs.delete(ma)
                  }
                })
              })
            }

            log(
              `received [${peerInfos.map((p) => chalk.blue(p.id.toB58String())).join(', ')}] from peer ${chalk.blue(
                peer
              )}`
            )
          } catch (err) {
            verbose('error querying peer', err)
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

        verbose('crawl complete')
        resolve()
      }

      // @TODO re-enable this once routing is done properly.
      // if (!isDone()) {
      //   throw Error(`Unable to find enough other nodes in the network.`)
      // }
    })
  }

  handleCrawlRequest(conn?: Connection) {
    verbose('crawl requested')
    return async function* (this: Crawler<Chain>) {
      const amountOfNodes = Math.min(CRAWLING_RESPONSE_NODES, this.node.network.peerStore.peers.length)

      if (this.options?.timeoutIntentionally) {
        await new Promise((resolve) => setTimeout(resolve, CRAWL_TIMEOUT + 100))
      }

      const selectedNodes = randomSubset(
        this.node.network.peerStore.peers,
        amountOfNodes,
        (entry: Entry) =>
          entry.id !== this.node.peerInfo.id.toB58String() &&
          (conn == null || entry.id !== conn.remotePeer.toB58String())
      ).map((peer) => {
        // convert peerId to peerInfo
        const found = this.node.peerStore.get(PeerId.createFromB58String(peer.id))
        const result = new PeerInfo(PeerId.createFromB58String(peer.id))
        if (found) {
          found.multiaddrs
            .toArray()
            .filter((ma) => shouldIncludePeerInCrawlResponse(ma, conn.remoteAddr))
            .forEach((ma) => result.multiaddrs.add(ma))
        }
        return result
      })

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

    log(
      `Crawling results:\n    ${chalk.yellow(`contacted nodes:`)}: ${contactedNodes}\n    ${chalk.green(
        `new nodes`
      )}: ${now - before} node${now - before == 1 ? '' : 's'}\n    total: ${now} node${now == 1 ? '' : 's'}`
    )
  }
}

export { Crawler }
