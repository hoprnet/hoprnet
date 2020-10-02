import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import chalk from 'chalk'
import { AbstractCommand } from './abstractCommand'

export default class ListConnectedPeers extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'listConnectedPeers'
  }

  public help() {
    return 'list the other connected HOPR nodes '
  }

  public async execute(): Promise<string | void> {
    let peers = Array.from(this.node.peerStore.peers.values())
    if (peers.length == 0) {
      return 'Not currently connected to any peers'
    }

    let idstr = peers.map((p) => chalk.green(p.id.toB58String()))
    return `Connected to: \n - ${idstr.join('\n - ')}\n`
  }
}
