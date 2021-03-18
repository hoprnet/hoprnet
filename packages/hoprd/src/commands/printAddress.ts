import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { AbstractCommand } from './abstractCommand'
import { styleValue } from './utils'

export default class PrintAddress extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
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
  public async execute(query: string): Promise<string> {
    const { utils } = this.node.paymentChannels

    const hoprPrefix = 'HOPR Address:'
    const hoprAddress = this.node.getId().toB58String()

    if (query.trim() === 'hopr') {
      return hoprAddress
    }

    // @TODO: use 'NativeBalance' and 'Balance' to display currencies
    const nativePrefix = 'ETH Address:'
    const nativeAddress = u8aToHex(await utils.pubKeyToAccountId(this.node.getId().pubKey.marshal()))

    if (query.trim() === 'native') {
      return nativeAddress
    }

    const prefixLength = Math.max(hoprPrefix.length, nativePrefix.length) + 2

    return [
      `${hoprPrefix.padEnd(prefixLength, ' ')}${styleValue(hoprAddress, 'peerId')}`,
      `${nativePrefix.padEnd(prefixLength, ' ')}${styleValue(nativeAddress, 'peerId')}`
    ].join('\n')
  }
}
