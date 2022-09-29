import type API from '../utils/api'
import { Command, type CacheFunctions } from '../utils/command'

export default class Version extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super({}, api, cache)
  }

  public name() {
    return 'version'
  }

  public description() {
    return 'Displays the version the HOPRd node is running.'
  }

  public async execute(log: (msg: string) => void, _query: string): Promise<void> {
    const response = await this.api.getVersion()
    if (!response.ok) return log(this.failedCommand('get version'))
    return log(await response.text())
  }
}
