import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '..'

import { Crawler } from './crawler'
import { Heartbeat } from './heartbeat'

class Network<Chain extends HoprCoreConnector> {
  public crawler: Crawler<Chain>
  public heartbeat: Heartbeat<Chain>

  constructor(node: Hopr<Chain>) {
    this.crawler = new Crawler(node)
    this.heartbeat = new Heartbeat(node)
  }
}

export { Network }
