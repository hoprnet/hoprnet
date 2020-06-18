import { keywords } from '../utils/keywords'

import chalk from 'chalk'

import AbstractCommand from './abstractCommand'

export default class ListCommands implements AbstractCommand {
  execute() {
    let maxLength = 0
    for (let i = 0; i < keywords.length; i++) {
      if (keywords[i][0].length > maxLength) {
        maxLength = keywords[i][0].length
      }
    }

    let str = ''
    for (let i = 0; i < keywords.length; i++) {
      str += chalk.yellow(('  ' + keywords[i][0]).padEnd(maxLength + 6, ' '))
      str += keywords[i][1]

      if (i < keywords.length - 1) {
        str += '\n'
      }
    }

    console.log(str)
  }

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
    cb(undefined, [[''], line])
  }
}
