import type { HoprOptions } from '..'
import type { Interactions } from '../interactions'
import type { LibP2P } from '../index'
import { Crawler } from './crawler'
import Heartbeat from './heartbeat'
import NetworkPeers from './network-peers'
import Stun from './stun'
import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'

class Network {
  public crawler: Crawler
  public heartbeat: Heartbeat
  public networkPeers: NetworkPeers
  public stun?: Stun

  constructor(node: LibP2P, interactions: Interactions<any>, options: HoprOptions) {
    const putPeer = (_ma: Multiaddr) => {
      /* No-op */
    }

    const getPeer = (id: PeerId): Multiaddr[] => {
      let addrs = node.peerStore.addressBook.get(id)
      return addrs?.map((a) => {
        if (!a.multiaddr.getPeerId()) {
          return a.multiaddr.encapsulate(`/p2p/${id.toB58String()}`)
        }
        return a.multiaddr
      })
    }

    this.networkPeers = new NetworkPeers(Array.from(node.peerStore.peers.values()).map((x) => x.id))
    this.heartbeat = new Heartbeat(this.networkPeers, interactions.network.heartbeat, node.hangUp.bind(node))
    this.crawler = new Crawler(node.peerId, this.networkPeers, interactions.network.crawler, getPeer, putPeer)

    if (options.bootstrapNode) {
      this.stun = new Stun(options.hosts)
    }
  }

  async start() {
    if (this.stun) {
      await this.stun?.startServer()
    }

    this.heartbeat?.start()
  }

  async stop() {
    if (this.stun) {
      await this.stun?.stopServer()
    }

    this.heartbeat?.stop()
  }
}

export { Network }
