import type { Interactions } from '../interactions'
import type { LibP2P } from '../index'
import Heartbeat from './heartbeat'
import NetworkPeers from './network-peers'

class Network {
  public heartbeat: Heartbeat
  public networkPeers: NetworkPeers

  constructor(node: LibP2P, interactions: Interactions<any>) {
    this.networkPeers = new NetworkPeers(Array.from(node.peerStore.peers.values()).map((x) => x.id))
    this.heartbeat = new Heartbeat(this.networkPeers, interactions.network.heartbeat, node.hangUp.bind(node))
  }

  async start() {
    this.heartbeat?.start()
  }

  async stop() {
    this.heartbeat?.stop()
  }
}

export { Network }
