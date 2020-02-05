import { HoprCoreConnectorInstance } from '@hoprnet/hopr-core-connector-interface'

import Hopr from '../..'

import { Crawler } from './crawler'

class NetworkInteractions<Chain extends HoprCoreConnectorInstance> {
  crawler: Crawler<Chain>

  constructor(node: Hopr<Chain>) {
    this.crawler = new Crawler(node)
  }
}

export { NetworkInteractions }
