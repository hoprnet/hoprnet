import type { HoprOptions } from '..'
import type { Interactions } from '../interactions'
import type { LibP2P } from '../index'
import { Crawler } from './crawler'
import Heartbeat from './heartbeat'
import NetworkPeers from './network-peers'
import Stun from './stun'
import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import PeerInfo from 'peer-info'

type TestOpts = {
  crawl?: { timeoutIntentionally?: boolean }
}
class Network {
  public crawler: Crawler
  public heartbeat: Heartbeat
  public networkPeers: NetworkPeers
  public stun?: Stun

  constructor(node: LibP2P, interactions: Interactions<any>, private options: HoprOptions, testingOptions?: TestOpts) {
    // These are temporary, and will be replaced by accessors to the addressBook
    const putPeer = (ma: Multiaddr) => {
      if (!ma.getPeerId()) {
        throw new Error('Cannot store a peer without an ID')
      }
      const pinfo = new PeerInfo(PeerId.createFromCID(ma.getPeerId()))
      pinfo.multiaddrs.add(ma)
      node.peerStore.put(pinfo)
    }

    const getPeer = (id: PeerId): Multiaddr[] => {
      let addrs = node.peerStore.get(id).multiaddrs.toArray()
      return addrs.map((a) => {
        if (!a.getPeerId()) {
          return a.encapsulate(`/p2p/${id.toB58String()}`)
        }
        return a
      })
    }

    this.networkPeers = new NetworkPeers(node.peerStore.peers.values())
    this.heartbeat = new Heartbeat(this.networkPeers, interactions.network.heartbeat, node.hangUp)
    this.crawler = new Crawler(
      node.peerInfo.id,
      this.networkPeers,
      interactions.network.crawler,
      getPeer,
      putPeer,
      testingOptions?.crawl
    )

    node.on('peer:connect', (peerInfo: PeerInfo) => {
      this.networkPeers.onPeerConnect(peerInfo)
      this.heartbeat.connectionListener(peerInfo)
    })

    if (options.bootstrapNode) {
      this.stun = new Stun(options.hosts)
    }
  }

  async start() {
    if (this.options.bootstrapNode) {
      await this.stun?.startServer()
    }

    this.heartbeat?.start()
  }

  async stop() {
    if (this.options.bootstrapNode) {
      await this.stun?.stopServer()
    }

    this.heartbeat?.stop()
  }
}

export { Network }
