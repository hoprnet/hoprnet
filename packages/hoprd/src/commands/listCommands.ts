import { AbstractCommand } from './abstractCommand.js'
import { getPaddingLength, styleValue } from './utils/index.js'

function getOptions(
  options: { value: any; description?: string }[],
  style: 'compact' | 'vertical' = 'compact'
): string[] {
  if (style === 'compact') {
    return [`Options: ${options.map((o) => String(o.value)).join('|')}`, '\n']
  } else {
    const padding = getPaddingLength(options.map((o) => String(o.value)))

    return [
      'Options:',
      ...options.map((option) => {
        return [
          // needed to preperly format the array
          '\n',
          '- ',
          styleValue(String(option.value).padEnd(padding), 'highlight'),
          option.description
        ].join('')
      }),
      '\n'
    ]
  }
}
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

  public execute(log) {
    return log(
      getOptions(
        this.getCommands()
          .filter((command) => !command.hidden)
          .map((command) => ({
            value: command.name(),
            description: command.help()
          }))
          .sort((a, b) => {
            return String(a.value).localeCompare(String(b.value))
          }),
        'vertical'
      ).join('')
    )
  }
}
