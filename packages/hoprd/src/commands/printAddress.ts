import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand.js'
import { styleValue } from './utils/index.js'
import { PublicKey } from '@hoprnet/hopr-utils'

export default class PrintAddress extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'address'
  }

  public help() {
    return 'Displays the native and HOPR addresses of this node'
  }

  /**
   * Prints the name of the network we are using and the
   * identity that we have on that chain.
   * @notice triggered by the CLI
   */
  public async execute(log, query: string): Promise<void> {
    const hoprPrefix = 'HOPR Address:'
    const hoprAddress = this.node.getId().toB58String()

    if (query.trim() === 'hopr') {
      return log(hoprAddress)
    }

    // @TODO: use 'NativeBalance' and 'Balance' to display currencies
    const nativePrefix = 'ETH Address:'
    const nativeAddress = new PublicKey(this.node.getId().pubKey.marshal()).toAddress().toHex()

    if (query.trim() === 'native') {
      return log(nativeAddress)
    }

    const prefixLength = Math.max(hoprPrefix.length, nativePrefix.length) + 2

    return log(
      [
        `${hoprPrefix.padEnd(prefixLength, ' ')}${styleValue(hoprAddress, 'peerId')}`,
        `${nativePrefix.padEnd(prefixLength, ' ')}${styleValue(nativeAddress, 'peerId')}`
      ].join('\n')
    )
  }
}
