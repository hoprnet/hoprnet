import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'
import { Logger } from '@hoprnet/hopr-utils'

const log = Logger.getLogger('hoprd.commands.stopNode')

export default class StopNode extends AbstractCommand {
  constructor(public node: Hopr) {
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
      log.info(`Ungracefully stopping node after timeout`)
      process.exit(0)
    }, 10 * 1000)

    try {
      await this.node.stop()
      clearTimeout(timeout)
      process.exit(0)
    } catch (error) {
      log.error('Error while killing process', error)
      return styleValue(error.message, 'failure')
    }
  }
}
