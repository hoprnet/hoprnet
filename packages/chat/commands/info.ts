import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from '../utils'
import Multiaddr from 'multiaddr'

export class Info extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'info'
  }

  public help() {
    return 'Information about the HOPR Node, including any options it started with'
  }

  public async execute(): Promise<string> {
    // @TODO Add connector info etc.
    return [
      `Bootstrap Servers: ${this.node.bootstrapServers.map((p) => styleValue(p.id.toB58String(), 'peerId'))}`,
      `Available at: ${this.node.getAddresses().map((ma: Multiaddr) => ma.toString())}`
    ].join('\n')
  }
}
