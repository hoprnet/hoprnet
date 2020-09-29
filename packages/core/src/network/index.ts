import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import type { HoprOptions } from '..'
import type PeerInfo from 'peer-info'

import { Crawler } from './crawler'
import Heartbeat from './heartbeat'
import PeerStore from './peerStore'
import Stun from './stun'

class Network<Chain extends HoprCoreConnector> {
  public crawler: Crawler
  public heartbeat: Heartbeat
  public peerStore: PeerStore
  public stun?: Stun

  constructor(node: Hopr<Chain>, private options: HoprOptions) {
    this.peerStore = new PeerStore(node.peerStore.peers.values())
    this.heartbeat = new Heartbeat(this.peerStore, node.interactions.network.heartbeat, node.hangUp)
    this.crawler = new Crawler(node.peerInfo.id, this.peerStore, node.interactions.network.crawler, node.peerStore)

    node.on('peer:connect',  (peerInfo: PeerInfo) => {
      this.peerStore.onPeerConnect(peerInfo)
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
