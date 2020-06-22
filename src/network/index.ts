import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import type { HoprOptions } from '..'

import { Crawler } from './crawler'
import Heartbeat from './heartbeat'
import PeerStore from './peerStore'
import Stun from './stun'

class Network<Chain extends HoprCoreConnector> {
  public crawler: Crawler<Chain>
  public heartbeat: Heartbeat<Chain>
  public peerStore: PeerStore<Chain>
  public stun?: Stun

  constructor(node: Hopr<Chain>, private options: HoprOptions) {
    this.crawler = new Crawler(node)
    this.heartbeat = new Heartbeat(node)
    this.peerStore = new PeerStore(node)

    if (options.bootstrapNode) {
      this.stun = new Stun(options)
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
