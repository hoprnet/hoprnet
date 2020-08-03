import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

import chalk from 'chalk'
import PeerId from 'peer-id'

import { isBootstrapNode } from '../utils'

import AbstractCommand from './abstractCommand'

export default class Crawl implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  /**
   * Crawls the network to check for other nodes. Triggered by the CLI.
   */
  async execute(): Promise<void> {
    try {
      await this.node.network.crawler.crawl(
        (peer: string) => !isBootstrapNode(this.node, PeerId.createFromB58String(peer))
      )
    } catch (err) {
      console.log(chalk.red(err.message))
    }
  }

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
    cb(undefined, [[''], line])
  }
}
