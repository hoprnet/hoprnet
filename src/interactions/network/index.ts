import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import Hopr from '../..'

import { Crawler } from './crawler'
import { Heartbeat } from './heartbeat'

class NetworkInteractions<Chain extends HoprCoreConnector> {
  crawler: Crawler<Chain>
  heartbeat: Heartbeat<Chain>

  constructor(node: Hopr<Chain>) {
    this.crawler = new Crawler(node)
    this.heartbeat = new Heartbeat(node)
  }
}

export { NetworkInteractions }
