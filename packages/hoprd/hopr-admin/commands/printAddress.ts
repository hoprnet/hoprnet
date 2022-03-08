import { AbstractCommand } from './abstractCommand'
import { getAddresses } from '../fetch'
import { styleValue } from './utils'

export default class PrintAddress extends AbstractCommand {
  constructor() {
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
    const addresses = await getAddresses()

    const hoprPrefix = 'HOPR Address:'
    const hoprAddress = addresses.hoprAddress

    if (query.trim() === 'hopr') {
      return log(hoprAddress)
    }

    // @TODO: use 'NativeBalance' and 'Balance' to display currencies
    const nativePrefix = 'ETH Address:'
    const nativeAddress = addresses.nativeAddress

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
