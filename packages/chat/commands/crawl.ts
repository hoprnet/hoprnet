import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import PeerId from 'peer-id'
import { isBootstrapNode, styleValue } from '../utils'
import { AbstractCommand } from './abstractCommand'

export default class Crawl extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'crawl'
  }

  public help() {
    return 'Crawls the network and tries to find other nodes'
  }

  /**
   * Crawls the network to check for other nodes. Triggered by the CLI.
   */
  public async execute(): Promise<string | void> {
    try {
      let info = await this.node.crawl((peer: PeerId) => !isBootstrapNode(this.node, peer))
      return `
        Crawled network, contacted ${styleValue(info.contacted.length)} peers. 
        Connected to ${styleValue(this.node.getConnectedPeers().length)} peers
      `
    } catch (err) {
      if (err && err.message) {
        return styleValue(err.message, 'failure')
      }
      return `Unknown error ${err}`
    }
  }
}
