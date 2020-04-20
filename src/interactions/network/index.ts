import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

import Hopr from '../..'

import { Crawler } from './crawler'
import { ForwardPacketInteraction } from './forwardPacket'
import { Heartbeat } from './heartbeat'

class NetworkInteractions<Chain extends HoprCoreConnector> {
  crawler: Crawler<Chain>
  forwardPacket: ForwardPacketInteraction<Chain>
  heartbeat: Heartbeat<Chain>

  constructor(node: Hopr<Chain>) {
    this.crawler = new Crawler(node)
    this.forwardPacket = new ForwardPacketInteraction(node)
    this.heartbeat = new Heartbeat(node)
  }
}

export { NetworkInteractions }
