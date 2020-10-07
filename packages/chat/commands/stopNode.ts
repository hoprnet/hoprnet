import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from '../utils'

export default class StopNode extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'quit'
  }

  public help() {
    return 'Stops the node and terminates the process'
  }
  /**
   * Stops the node and kills the process in case it does not quit by itself.
   */
  public async execute(): Promise<string | void> {
    const timeout = setTimeout(() => {
      console.log(`Ungracefully stopping node after timeout.`)
      process.exit(0)
    }, 10 * 1000)

    try {
      await this.node.stop()
      clearTimeout(timeout)
      process.exit(0)
    } catch (error) {
      return styleValue(error.message, 'failure')
    }
  }
}
