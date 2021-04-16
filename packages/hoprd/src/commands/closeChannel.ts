import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import chalk from 'chalk'
import { AbstractCommand, GlobalState } from './abstractCommand'
import { checkPeerIdInput, styleValue } from './utils'
import { Logger } from '@hoprnet/hopr-utils'

const log: Logger = Logger.getLogger('hoprd.commands.closeChannel')

export default class CloseChannel extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'close'
  }

  public help() {
    return 'Close an open channel'
  }

  async execute(query: string, state: GlobalState): Promise<string | void> {
    if (query == null) {
      return styleValue(`Invalid arguments. Expected 'close <peerId>'. Received '${query}'`, 'failure')
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query, state)
    } catch (err) {
      log.error('Error while checking peerId', err)
      return styleValue(err.message, 'failure')
    }

    try {
      const { status, receipt } = await this.node.closeChannel(peerId)

      if (status === 'PENDING') {
        return `${chalk.green(`Closing channel. Receipt: ${styleValue(receipt, 'hash')}`)}.`
      } else {
        return `${chalk.green(
          `Initiated channel closure, the channel must remain open for at least 2 minutes. Please send the close command again once the cool-off has passed. Receipt: ${styleValue(
            receipt,
            'hash'
          )}`
        )}.`
      }
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }
}
