import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'

import chalk from 'chalk'

import { u8aToHex } from '@hoprnet/hopr-utils'

export default class PrintAddress extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }
  name() { return 'myAddress'}
  help() { return 'shows the address of this node' }

  /**
   * Prints the name of the network we are using and the
   * identity that we have on that chain.
   * @notice triggered by the CLI
   */
  async execute(): Promise<string> {
    const prefixLength = Math.max(this.node.paymentChannels.constants.CHAIN_NAME.length, 'HOPR'.length) + 3

    return `${(this.node.paymentChannels.constants.CHAIN_NAME + ':').padEnd(prefixLength, ' ')}${chalk.green(
        u8aToHex(await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()))
      )}\n` +
        /* prettier-ignore */
        `${'HOPR:'.padEnd(prefixLength, ' ')}${chalk.green(this.node.peerInfo.id.toB58String())}`
  }
}
