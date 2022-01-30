import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import chalk from 'chalk'
import type { StateOps } from '../types'
import { AbstractCommand } from './abstractCommand'
import { checkPeerIdInput, styleValue } from './utils'
import { ChannelStatus } from '@hoprnet/hopr-utils'

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

  async execute(log, query: string, { getState }: StateOps): Promise<void> {
    if (query == null) {
      return log(styleValue(`Invalid arguments. Expected 'close <peerId>'. Received '${query}'`, 'failure'))
    }

    let peerId: PeerId
    try {
      peerId = checkPeerIdInput(query, getState())
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    log('Closing channel...')

    try {
      const { status, receipt } = await this.node.closeChannel(peerId)
      const smartContractInfo = this.node.smartContractInfo()
      const channelClosureMins = Math.ceil(smartContractInfo.channelClosureSecs / 60) // convert to minutes

      if (status === ChannelStatus.PendingToClose) {
        return log(`${chalk.green(`Closing channel. Receipt: ${styleValue(receipt, 'hash')}`)}.`)
      } else {
        return log(
          `${chalk.green(
            `Initiated channel closure, the channel must remain open for at least ${channelClosureMins} minutes. Please send the close command again once the cool-off has passed. Receipt: ${styleValue(
              receipt,
              'hash'
            )}`
          )}.`
        )
      }
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }
  }
}
