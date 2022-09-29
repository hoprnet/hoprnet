import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command, type CacheFunctions } from '../utils/command'

export default class Help extends Command {
  constructor(api: API, cache: CacheFunctions, private commands: Command[]) {
    super(
      {
        default: [[], 'displays help'],
        showAll: [[['constant', 'all']], 'shows hidden commands']
      },
      api,
      cache
    )
  }

  public name() {
    return 'help'
  }

  public description() {
    return 'Displays all the command options.'
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , param = false] = this.assertUsage(query)
    if (error) return log(error)

    const showAll = param === 'all'

    const arr = this.commands
      .filter((cmd) => {
        if (showAll) return true
        return !cmd.hidden
      })
      .map<[string, string]>((cmd) => [cmd.name(), cmd.description()])
      // Sort commands alphabetically
      .sort((a, b) => a[0].localeCompare(b[0], 'en'))

    if (!showAll) arr.push(['', "* Display hidden commands by running 'help all'"])

    return log(toPaddedString(arr))
  }
}
