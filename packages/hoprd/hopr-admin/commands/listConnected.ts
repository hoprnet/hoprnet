import { AbstractCommand } from './abstractCommand'

export default class ListConnectedPeers extends AbstractCommand {
  constructor() {
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
