import type PeerId from 'peer-id'
import chalk from 'chalk'
import { AbstractCommand } from './abstractCommand'
import { checkPeerIdInput, styleValue } from './utils'
import { closeChannel, getNodeInfo } from '../fetch'
import { ChannelStatus } from './utils/util'

export default class CloseChannel extends AbstractCommand {
  constructor() {
    super()
  }

  public name() {
    return 'close'
  }

  public help() {
    return 'Close an open channel'
  }

  async execute(log, query: string): Promise<void> {
    if (query == null) {
      return log(styleValue(`Invalid arguments. Expected 'close <peerId>'. Received '${query}'`, 'failure'))
    }

    let peerId: PeerId
    try {
      peerId = checkPeerIdInput(query)
    } catch (err) {
      return log(styleValue(err.message, 'failure'))
    }

    log('Closing channel...')

    try {
      const response = await closeChannel(peerId.toB58String())
      const nodeInfo = await getNodeInfo()


      if (response.status === ChannelStatus.PendingToClose) {
        return log(`${chalk.green(`Closing channel. Receipt: ${styleValue(response.receipt, 'hash')}`)}.`)
      } else {
        return log(
          `${chalk.green(
            `Initiated channel closure, the channel must remain open for at least ${nodeInfo.channelClosurePeriod} minutes. Please send the close command again once the cool-off has passed. Receipt: ${styleValue(
              response.receipt,
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
