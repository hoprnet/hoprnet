import { AbstractCommand, GlobalState, AutoCompleteResult } from './abstractCommand'
import { checkPeerIdInput, getPeersIdsAsString } from '../utils'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import chalk from 'chalk'

export class Info extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() {
    return 'info'
  }
  help() {
    return 'Information about the HOPR Node, including any options it started with'
  }

  async execute(query: string, settings: GlobalState): Promise<string | void> {
    // TODO Add connector info etc.
    return `
      Bootstrap Servers: ${this.node.bootstrapServers.map((p) => chalk.green(p.id.toB58String()))}
    `
  }
}
