import { AbstractCommand } from './abstractCommand'
import HoprFetcher from '../fetch'

export default class ListConnectedPeers extends AbstractCommand {
  constructor(fetcher: HoprFetcher) {
    super(fetcher)
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
