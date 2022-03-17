import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command } from '../utils/command'

export default class Addresses extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[], 'shows all addresses'],
        onlyOne: [[['hoprOrNative', 'type', false]], 'shows one address']
      },
      api,
      extra
    )
  }

  public name() {
    return 'address'
  }

  public description() {
    return 'Displays the native and HOPR addresses of this node, optionally view one address.'
  }

  /**
   * Prints the name of the network we are using and the
   * identity that we have on that chain.
   * @notice triggered by the CLI
   */
  public async execute(log, query: string): Promise<void> {
    const [error, use, type] = this.assertUsage(query) as [string | undefined, string, string]
    if (error) return log(error)

    const addresses = await this.api.getAddresses()

    const hoprPrefix = 'HOPR Address:'
    const hoprAddress = addresses.hopr
    const nativePrefix = 'ETH Address:'
    const nativeAddress = addresses.native

    if (use === 'onlyOne') {
      if (type === 'hopr') {
        return log(hoprAddress)
      } else if (type === 'native') {
        return log(nativeAddress)
      }
    } else {
      return log(
        toPaddedString([
          [hoprPrefix, hoprAddress],
          [nativePrefix, nativeAddress]
        ])
      )
    }
  }
}
