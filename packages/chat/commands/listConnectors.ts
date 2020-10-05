import { AbstractCommand } from './abstractCommand'

import chalk from 'chalk'
import { knownConnectors } from '../utils/knownConnectors'

export default class ListConnectors extends AbstractCommand {
  name() {
    return 'listConnectors'
  }
  help() {
    return 'lists all installed blockchain connectors'
  }
  /**
   * Check which connectors are present right now.
   * @notice triggered by the CLI
   */
  async execute(): Promise<string> {
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
      return str
    } else {
      return chalk.red(`Could not find any connectors. Please make sure there is one available in 'node_modules'!`)
    }
  }
}
