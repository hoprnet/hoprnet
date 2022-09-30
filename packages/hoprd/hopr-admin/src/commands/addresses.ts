import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command, type CacheFunctions, type HoprOrNative } from '../utils/command'

export default class Addresses extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[], 'shows all addresses'],
        onlyOne: [[['hoprOrNative']], 'shows one address']
      },
      api,
      cache
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
  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, use, type] = this.assertUsage(query) as [string | undefined, string, HoprOrNative]
    if (error) return log(error)

    const response = await this.api.getAddresses()

    if (!response.ok) {
      return log(
        await this.failedApiCall(response, 'fetch addresses', {
          422: (v) => v.error
        })
      )
    }

    const addresses = await response.json()
    const symbols = this.cache.getSymbols()

    const hoprPrefix = `${symbols.hoprDisplay} Address:`
    const hoprAddress = addresses.hopr
    const nativePrefix = `${symbols.nativeDisplay} Address:`
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
