import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command } from '../utils/command'

export default class Help extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }, private commands: Command[]) {
    super(
      {
        default: [[], 'displays help'],
        showAll: [[['boolean', 'show hidden commands', true]], 'includes hidden commands']
      },
      api,
      extra
    )
  }

  public name() {
    return 'help'
  }

  public description() {
    return 'Displays all the command options.'
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, showHidden] = this.assertUsage(query)
    if (error) return log(error)

    log(
      toPaddedString(
        this.commands
          .filter((cmd) => {
            if (showHidden) return true
            return !cmd.hidden
          })
          .map<[string, string]>((cmd) => [cmd.name(), cmd.description()])
      )
    )
  }
}
