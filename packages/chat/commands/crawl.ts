import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

import chalk from 'chalk'
import PeerId from 'peer-id'

import { isBootstrapNode } from '../utils'
import { AbstractCommand } from './abstractCommand'

export default class Crawl extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() {
    return 'crawl'
  }

  help() {
    return 'crawls the network and tries to find other nodes'
  }

  /**
   * Crawls the network to check for other nodes. Triggered by the CLI.
   */
  async execute(): Promise<string | void> {
    try {
      await this.node.network.crawler.crawl(
        (peer: string) => !isBootstrapNode(this.node, PeerId.createFromB58String(peer))
      )
      return `Crawled network, connected to ${this.node.peerStore.peers.size} peers`
    } catch (err) {
      chalk.red(err.message)
    }
  }
}
