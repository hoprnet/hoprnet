import chalk from 'chalk'
import AbortController from 'abort-controller'
import { getTokens } from '../utils'
import { randomInteger } from '@hoprnet/hopr-utils'
import type { Token } from '../utils'
import { MAX_HOPS, CRAWLING_RESPONSE_NODES } from '../constants'
import { CrawlResponse, CrawlStatus } from '../messages'
import PeerId from 'peer-id'
import type { Connection } from 'libp2p'
import type { Entry } from './network-peers'
import NetworkPeerStore from './network-peers'
import { peerHasOnlyPublicAddresses, isOnPrivateNet /*, PRIVATE_NETS */ } from '../filters'
import debug from 'debug'
import Multiaddr from 'multiaddr'
import { Crawler as CrawlInteraction } from '../interactions/network/crawler'

const log = debug('hopr-core:crawler')
const verbose = debug('hopr-core:verbose:crawler')
const blue = chalk.blue

const MAX_PARALLEL_REQUESTS = 7
export const CRAWL_TIMEOUT = 2 * 1000

export type CrawlInfo = {
  contacted: PeerId[]
  errors: (Error | string)[]
}

let stringToPeerId = (s: string): PeerId => {
  return PeerId.createFromB58String(s)
}

export const shouldIncludePeerInCrawlResponse = (_peer: Multiaddr, _them: Multiaddr): boolean => {
  // We are being requested a crawl from a node that is on a remote network, so
  // it does not benefit them for us to give them addresses on our private
  // network, therefore let's first filter these out.
  // if (
  //   ['ip4', 'ip6', 'dns4', 'dns6'].includes(them.protoNames()[0]) &&
  //   !them.nodeAddress().address.match(PRIVATE_NETS) &&
  //   isOnPrivateNet(peer)
  // ) {
  //   verbose('rejecting peer from crawl results as it only has private addresses, and the requesting node is remote')
  //   return false // Reject peer
  // }
  return true
}

class Crawler {
  constructor(
    private id: PeerId,
    private networkPeers: NetworkPeerStore,
    private crawlInteraction: CrawlInteraction,
    private getPeer: (peer: PeerId) => Multiaddr[],
    private putPeer: (ma: Multiaddr) => void,
    private options?: {
      timeoutIntentionally?: boolean
    }
  ) {}

  /**
   *
   * @param filter
   */
  crawl(filter?: (peer: PeerId) => boolean): Promise<CrawlInfo> {
    verbose('creating a crawl')
    return new Promise(async (resolve) => {
      let aborted = false

      const errors: Error[] = []

      // fast non-inclusion check
      const contactedPeerIds = new Set<string>()

      let unContactedPeers: string[] = [] // @TODO add new peerIds lazily

      let before = 0 // number of peers before crawling
      let current = 0 // current number of peers

      const abort = new AbortController()

      const timeout = setTimeout(() => {
        aborted = true
        verbose('aborting crawl due to timeout')
        abort.abort()
        this.printStatsAndErrors(contactedPeerIds, errors, current, before)
        resolve({
          contacted: Array.from(contactedPeerIds.values()).map((x) => stringToPeerId(x)),
          errors
        })
      }, CRAWL_TIMEOUT)

      log(`Crawling started`)

      this.networkPeers.cleanupBlacklist()

      unContactedPeers.push(...this.networkPeers.peers.map((entry: Entry) => entry.id.toB58String()))
      verbose(`added ${unContactedPeers.length} peers to crawl list`)

      if (filter != null) {
        for (let i = 0; i < unContactedPeers.length; i++) {
          if (filter(stringToPeerId(unContactedPeers[i]))) {
            before += 1
          }
        }
        verbose('filter passed; number of peers before: ', before)
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
      const removeRandomNode = (): PeerId => {
        if (unContactedPeers.length == 0) {
          throw Error(`Cannot pick a random node because there are none.`)
        }

        const index = randomInteger(0, unContactedPeers.length)

        if (index == unContactedPeers.length - 1) {
          return stringToPeerId(unContactedPeers.pop())
        }

        const selected = unContactedPeers[index]
        unContactedPeers[index] = unContactedPeers.pop()

        return stringToPeerId(selected)
      }

      /**
       * Stores the crawling "threads"
       */
      const promises: Promise<void>[] = Array.from({ length: MAX_PARALLEL_REQUESTS })

      /**
       * Connect to another peer and returns a promise that resolves to all received nodes
       * that were previously unknown.
       */
      const queryNode = async (peer: PeerId, token: Token): Promise<void> => {
        let addresses: Multiaddr[]

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
          contactedPeerIds.add(peer.toB58String())

          try {
            log(`querying ${blue(peer.toB58String())}`)
            addresses = await this.crawlInteraction.interact(peer, {
              signal: abort.signal
            })

            const addrs = this.getPeer(peer)
            if (addrs && peerHasOnlyPublicAddresses(addrs)) {
              // The node we are connecting to is on a remote network
              // and gives us addresses on a private network, then they are
              // not going to work for us. We should filter these out when we are
              // requested for a crawl, but in this instance they have given us
              // some anyway.
              addresses = addresses.filter((ma) => !isOnPrivateNet(ma))
            }

            log(
              `received [${addresses.map((p: Multiaddr) => blue(p.getPeerId())).join(', ')}] from peer ${blue(
                peer.toB58String()
              )}`
            )
          } catch (err) {
            verbose('error querying peer', err)
            addresses = []
            errors.push(err)
            continue
          }

          for (let i = 0; i < addresses.length; i++) {
            if (!addresses[i].getPeerId()) {
              throw Error('address does not contain peer id: ' + addresses[i].toString())
            }
            const peer = PeerId.createFromCID(addresses[i].getPeerId())

            if (peer.equals(this.id)) {
              continue
            }

            if (
              !contactedPeerIds.has(peer.toB58String()) &&
              !unContactedPeers.find((unContactedPeer: string) => unContactedPeer === peer.toB58String())
            ) {
              unContactedPeers.push(peer.toB58String())

              let beforeInserting = this.networkPeers.length
              this.networkPeers.push({
                id: peer,
                lastSeen: 0
              })

              if (filter == null || filter(peer)) {
                current = current + this.networkPeers.length - beforeInserting
              }
              this.putPeer(addresses[i])
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
        resolve({
          contacted: Array.from(contactedPeerIds.values()).map((x: string) => stringToPeerId(x)),
          errors
        })
      }

      // @TODO re-enable this once routing is done properly.
      // if (!isDone()) {
      //   throw Error(`Unable to find enough other nodes in the network.`)
      // }
    })
  }

  async answerCrawl(callerId: PeerId, callerAddress: Multiaddr): Promise<Multiaddr[]> {
    if (this.options?.timeoutIntentionally) {
      await new Promise((resolve) => setTimeout(resolve, CRAWL_TIMEOUT + 100))
    }

    return this.networkPeers
      .randomSubset(CRAWLING_RESPONSE_NODES, (id: PeerId) => !id.equals(this.id) && !id.equals(callerId))
      .map(this.getPeer) // NB: Multiple addrs per peer.
      .flat()
      .filter((ma: Multiaddr) => shouldIncludePeerInCrawlResponse(ma, callerAddress))
  }

  async *handleCrawlRequest(this: Crawler, conn: Connection) {
    verbose('crawl requested')
    const selectedNodes = await this.answerCrawl(conn.remotePeer, conn.remoteAddr)
    if (selectedNodes.length > 0) {
      yield new CrawlResponse(undefined, {
        status: CrawlStatus.OK,
        addresses: selectedNodes
      })
    } else {
      yield new CrawlResponse(undefined, {
        status: CrawlStatus.FAIL
      })
    }
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
    contactedPeerIds.forEach((p: string) => {
      contactedNodes += `\n        ${p}`
    })

    log(
      `Crawling results:\n    ${chalk.yellow(`contacted nodes:`)}: ${contactedNodes}\n    ${chalk.green(
        `new nodes`
      )}: ${now - before} node${now - before == 1 ? '' : 's'}\n    total: ${now} node${now == 1 ? '' : 's'}`
    )
  }
}

export { Crawler }
