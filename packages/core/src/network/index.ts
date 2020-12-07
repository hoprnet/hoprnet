import type { HoprOptions } from '..'
import type { Interactions } from '../interactions'
import type { LibP2P } from '../index'
import Heartbeat from './heartbeat'
import NetworkPeers from './network-peers'
import Stun from './stun'

class Network {
  public heartbeat: Heartbeat
  public networkPeers: NetworkPeers
  public stun?: Stun

  constructor(node: LibP2P, interactions: Interactions<any>, options: HoprOptions) {
    this.networkPeers = new NetworkPeers(Array.from(node.peerStore.peers.values()).map((x) => x.id))
    this.heartbeat = new Heartbeat(this.networkPeers, interactions.network.heartbeat, node.hangUp.bind(node))

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
