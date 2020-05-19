import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'
import type { HoprOptions } from '..'



import { Crawler } from './crawler'
import { Heartbeat } from './heartbeat'
import { StunServer } from './stun'

class Network<Chain extends HoprCoreConnector> {
  public crawler: Crawler<Chain>
  public heartbeat?: Heartbeat<Chain>
  public stun?: StunServer

  constructor(node: Hopr<Chain>, options: HoprOptions) {
    this.crawler = new Crawler(node)
    this.heartbeat = new Heartbeat(node)

    if (options.bootstrapNode) {
      this.stun = new StunServer(options)
    }
  }

  async start() {
    // await this.stun?.start()

    this.heartbeat?.start()
  }

  async stop() {
    // await this.stun?.stop()

    this.heartbeat?.stop()
  }
}

export { Network }
