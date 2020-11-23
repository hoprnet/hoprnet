import Heap from 'heap-js'
import { limitConcurrency, timeoutAfter } from '@hoprnet/hopr-utils'
import { CRAWLING_RESPONSE_NODES, MAX_PARALLEL_CONNECTIONS, CRAWL_FAIL_TIMEOUT, CRAWL_MAX_SIZE } from '../constants'
import PeerId from 'peer-id'
import NetworkPeerStore from './network-peers'
import { peerHasOnlyPublicAddresses, isOnPrivateNet, PRIVATE_NETS } from '../filters'
import debug from 'debug'
import Multiaddr from 'multiaddr'
import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import { Crawler as CrawlInteraction } from '../interactions/network/crawler'

const log = debug('hopr-core:crawler')

export type CrawlInfo = {
  contacted: PeerId[]
  errors: (Error | string)[]
}

type CrawlEdge = [PeerId, Number] // ID, weight
const has = (queue: Heap<CrawlEdge>, peer) => queue.contains(peer, (e) => e[0].equals(peer))

export const shouldIncludePeerInCrawlResponse = (peer: Multiaddr, them: Multiaddr): boolean => {
  // We are being requested a crawl from a node that is on a remote network, so
  // it does not benefit them for us to give them addresses on our private
  // network, therefore let's first filter these out.
  if (
    ['ip4', 'ip6', 'dns4', 'dns6'].includes(them.protoNames()[0]) &&
    !them.nodeAddress().address.match(PRIVATE_NETS) &&
    isOnPrivateNet(peer)
  ) {
    // rejecting peer from crawl as it has only private addresses,
    // and the requesting node is remote
    return false
  }
  return true
}

const compareWeight = (a, b) => b[1] - a[1]

class Crawler {
  constructor(
    private id: PeerId,
    private networkPeers: NetworkPeerStore,
    private crawlInteraction: CrawlInteraction,
    private indexer: Indexer,
    private getPeer: (peer: PeerId) => Multiaddr[],
    private putPeer: (ma: Multiaddr) => void,
    private stringToPeerId: (id: string) => PeerId = (s) => PeerId.createFromB58String(s) // TODO for testing
  ) {}

  private async weight(p: PeerId): Promise<CrawlEdge> {
    const peerEdges = await this.indexer.getChannelsFromPeer(p)
    const outgoingStake = peerEdges.reduce((x: IndexerChannel, y: IndexerChannel) => x[2] + y[2], 0)
    return [p, outgoingStake * Math.random()]
  }
  /**
   * @param filter
   */
  async crawl(filter: (peer: PeerId) => boolean = () => true): Promise<CrawlInfo> {
    const errors: Error[] = []
    const contacted = new Set<string>()
    let queue = new Heap<CrawlEdge>(compareWeight)
    queue.addAll(
      await Promise.all(
        this.networkPeers
          .all()
          .filter(filter)
          .map(async (p) => this.weight(p))
      )
    )
    const before = queue.length // number of peers before crawling

    log(`Crawling started`)
    const isDone = () => contacted.size >= CRAWL_MAX_SIZE || queue.length == 0

    const queryNode = async (abortSignal): Promise<void> => {
      let peer = queue.pop()[0]
      contacted.add(peer.toB58String())
      try {
        log(`querying ${peer.toB58String()}`)
        let addresses = await this.crawlInteraction.interact(peer, {
          signal: abortSignal
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

        log(`received [${addresses.map((p: Multiaddr) => p.getPeerId()).join(', ')}] from peer ${peer.toB58String()}`)

        for (let i = 0; i < addresses.length; i++) {
          if (!addresses[i].getPeerId()) {
            throw Error('address does not contain peer id: ' + addresses[i].toString())
          }
          const peer = this.stringToPeerId(addresses[i].getPeerId())

          if (peer.equals(this.id) || contacted.has(peer.toB58String()) || !filter(peer) || has(queue, peer)) {
            log('skipping', peer.toB58String())
            continue
          }
          queue.push(await this.weight(peer))
          this.putPeer(addresses[i])
          this.networkPeers.register(peer)
          log('adding to queue', peer.toB58String())
        }
      } catch (err) {
        log('error querying peer', err)
        errors.push(err)
      }
    }

    try {
      await timeoutAfter(
        (abortSignal) => limitConcurrency(MAX_PARALLEL_CONNECTIONS, isDone, () => queryNode(abortSignal)),
        CRAWL_FAIL_TIMEOUT
      )
    } catch (e) {
      log('Error', e)
      // timeouts are ok
    }

    this.debugStats(contacted, errors, contacted.size, before)
    log('crawl complete')
    return {
      contacted: Array.from(contacted.values()).map((x: string) => this.stringToPeerId(x)),
      errors
    }
  }

  public async answerCrawl(callerAddress: Multiaddr): Promise<Multiaddr[]> {
    return this.networkPeers
      .randomSubset(
        CRAWLING_RESPONSE_NODES,
        (id: PeerId) => !id.equals(this.id) && !id.equals(this.stringToPeerId(callerAddress.getPeerId()))
      )
      .map(this.getPeer) // NB: Multiple addrs per peer.
      .flat()
      .filter((ma: Multiaddr) => shouldIncludePeerInCrawlResponse(ma, callerAddress))
  }

  private debugStats(contactedPeerIds: Set<string>, errors: Error[], now: number, before: number) {
    log(`Crawling results:\n- contacted nodes: ${contactedPeerIds.size}\n- new nodes: ${now - before}\n- total: ${now}`)
    log('Contacted:')
    contactedPeerIds.forEach((p) => log('- ', p))
    if (errors.length > 0) {
      log(`Errors while crawling`)
      errors.forEach((e) => log(' - ', e.message))
    }
  }
}

export { Crawler }
