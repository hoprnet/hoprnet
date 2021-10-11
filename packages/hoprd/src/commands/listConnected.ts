import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand.js'

export default class ListConnectedPeers extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'peers'
  }

  public help() {
    return 'Lists connected and interesting HOPR nodes'
  }

  public async execute(log): Promise<void> {
    return log(await this.node.connectionReport())
  }
}
