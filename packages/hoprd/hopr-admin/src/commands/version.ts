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

  public async execute(log): Promise<void> {
    const version = await this.api.getVersion()
    return log(version)
  }
}
