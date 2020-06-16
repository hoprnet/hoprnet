import AbstractCommand from './abstractCommand'

import chalk from 'chalk'
import { knownConnectors } from '..'

export default class ListConnectors implements AbstractCommand {
  /**
   * Check which connectors are present right now.
   * @notice triggered by the CLI
   */
  async execute(): Promise<void> {
    let str = 'Available connectors:'
    let found = 0

    const promises = []
    for (let i = 0; i < knownConnectors.length; i++) {
      promises.push(
        import(knownConnectors[i][0]).then(
          () => {
            found++
            str += `\n  ${chalk.yellow(knownConnectors[i][0])} ${chalk.gray('=>')} ./hopr -n ${chalk.green(
              knownConnectors[i][1]
            )}`
          },
          () => {}
        )
      )
    }

    await Promise.all(promises)

    if (found > 0) {
      console.log(str)
    } else {
      console.log(
        chalk.red(`Could not find any connectors. Please make sure there is one available in 'node_modules'!`)
      )
    }
  }

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
    cb(undefined, [[''], line])
  }
}
