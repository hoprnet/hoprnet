import { AbstractCommand } from './abstractCommand'
import { getOptions } from '../utils'

export default class ListCommands extends AbstractCommand {
  constructor(private getCommands: () => AbstractCommand[]) {
    super()
  }

  public name() {
    return 'help'
  }

  public help() {
    return 'shows this help page'
  }

  public execute(): string {
    return getOptions(
      this.getCommands().map((command) => ({
        value: command.name(),
        description: command.help(),
      })),
      'vertical'
    ).join('')
  }
}
