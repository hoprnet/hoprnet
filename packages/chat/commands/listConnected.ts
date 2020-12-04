import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'

export default class ListConnectedPeers extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'peers'
  }

  public help() {
    return 'Lists connected and interesting HOPR nodes'
  }

  public async execute(): Promise<string | void> {
    return this.node.connectionReport()
  }
}
