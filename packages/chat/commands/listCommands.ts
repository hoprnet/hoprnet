import chalk from 'chalk'

import { AbstractCommand } from './abstractCommand'

export default class ListCommands extends AbstractCommand {
  constructor(private getCommands: () => AbstractCommand[]){
    super()
  }

  name(){
    return 'help'
  }

  help(){
    return 'shows this help page'
  }

  execute(): string {
    let names = this.getCommands().map(x => x.name())
    let helps = this.getCommands().map(x => x.help())

    let maxLength = Math.max(...names.map(x => x.length))

    let str = ''
    for (let i = 0; i < names.length; i++) {
      str += chalk.yellow(('  ' + names[i]).padEnd(maxLength + 6, ' '))
      str += helps[i]

      if (i < names.length - 1) {
        str += '\n'
      }
    }

    return str
  }
}
