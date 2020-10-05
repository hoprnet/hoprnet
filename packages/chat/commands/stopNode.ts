import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

import chalk from 'chalk'

import { AbstractCommand } from './abstractCommand'

export default class StopNode extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() {
    return 'quit'
  }
  help() {
    return 'stops the node and terminates the process'
  }
  /**
   * Stops the node and kills the process in case it does not quit by itself.
   */
  async execute(): Promise<void> {
    const timeout = setTimeout(() => {
      console.log(`Ungracefully stopping node after timeout.`)
      process.exit(0)
    }, 10 * 1000)

    try {
      await this.node.stop()
      clearTimeout(timeout)
      process.exit(0)
    } catch (err) {
      console.log(chalk.red(err.message))
    }
  }
}
