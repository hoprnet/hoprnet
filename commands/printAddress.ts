import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type AbstractCommand from './abstractCommand'

import chalk from 'chalk'

import { u8aToHex } from '@hoprnet/hopr-utils'

export default class PrintAddress implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  /**
   * Prints the name of the network we are using and the
   * identity that we have on that chain.
   * @notice triggered by the CLI
   */
  async execute(): Promise<void> {
    const prefixLength = Math.max(this.node.paymentChannels.constants.CHAIN_NAME.length, 'HOPR'.length) + 3

    console.log(
      `${(this.node.paymentChannels.constants.CHAIN_NAME + ':').padEnd(prefixLength, ' ')}${chalk.green(
        u8aToHex(await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()))
      )}\n` +
        /* prettier-ignore */
        `${'HOPR:'.padEnd(prefixLength, ' ')}${chalk.green(this.node.peerInfo.id.toB58String())}`
    )
  }

  complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void {
    cb(undefined, [[''], line])
  }
}
