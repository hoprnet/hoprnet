import type API from '../utils/api'
import { Command } from '../utils/command'

export default class Version extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super({}, api, extra)
  }

  public name() {
    return 'version'
  }

  public description() {
    return 'Displays the version the HOPRd node is running.'
  }

  public async execute(log: (msg: string) => void, _query: string): Promise<void> {
    const response = await this.api.getVersion()
    if (!response.ok) return log(this.invalidResponse('get version'))
    return log(await response.text())
  }
}
