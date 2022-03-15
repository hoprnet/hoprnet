import { AbstractCommand } from './abstractCommand'
import HoprFetcher from '../fetch'

export default class Version extends AbstractCommand {
  constructor(fetcher: HoprFetcher) {
    super(fetcher)
  }

  public name() {
    return 'version'
  }

  public help() {
    return 'Displays the version'
  }

  public async execute(log): Promise<void> {
    await this.hoprFetcher.getNodeVer().then((version) => log(version))
  }
}
