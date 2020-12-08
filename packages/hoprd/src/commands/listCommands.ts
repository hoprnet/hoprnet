import { AbstractCommand } from './abstractCommand'
import { getOptions } from './utils'

export default class ListCommands extends AbstractCommand {
  constructor(private getCommands: () => AbstractCommand[]) {
    super()
  }

  public name() {
    return 'help'
  }

  public help() {
    return 'Displays all the command options'
  }

  public execute(): string {
    return getOptions(
      this.getCommands()
        .map((command) => ({
          value: command.name(),
          description: command.help()
        }))
        .sort((a, b) => {
          return String(a.value).localeCompare(String(b.value))
        }),
      'vertical'
    ).join('')
  }
}
