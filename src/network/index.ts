import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import { Crawler } from './crawler'

import Hopr from '..'

class Network<Chain extends HoprCoreConnector> {
  public crawler: Crawler<Chain>

  constructor(node: Hopr<Chain>) {
    this.crawler = new Crawler(node)
  }
}

export { Network }
