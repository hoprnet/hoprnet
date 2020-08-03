import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

import chalk from 'chalk'

import type AbstractCommand from './abstractCommand'

export default class StopNode implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

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

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
    cb(undefined, [[''], line])
  }
}
