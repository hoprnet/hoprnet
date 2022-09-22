import type { PeerId } from '@libp2p/interface-peer-id'
import type API from '../utils/api'
import BN from 'bn.js'
import { utils as ethersUtils } from 'ethers'
import { Command, type CacheFunctions } from '../utils/command'

export default class OpenChannel extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[['hoprAddressOrAlias'], ['number', 'Amount of HOPR tokens']], 'opens channel']
      },
      api,
      cache
    )
  }

  public name() {
    return 'open'
  }

  public description() {
    return 'Opens a payment channel between you and the counterparty provided'
  }

  /**
   * Encapsulates the functionality that is executed once the user decides to open a payment channel
   * with another party.
   * @param query peerId string to send message to
   */
  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , counterparty, amount] = this.assertUsage(query) as [string | undefined, string, PeerId, number]
    if (error) return log(error)

    const amountToFund = new BN(String(ethersUtils.parseEther(String(amount))))
    const counterpartyStr = counterparty.toString()

    const balancesRes = await this.api.getBalances()
    if (!balancesRes.ok) {
      return log(
        await this.failedApiCall(balancesRes, `fetch balances so we can open channel ${counterpartyStr}`, {
          422: (v) => v.error
        })
      )
    }

    const myAvailableTokens = await balancesRes.json().then((d) => new BN(d.hopr))

    if (amountToFund.lten(0)) {
      return log(`Invalid 'amount' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens)) {
      return log(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toString(10)}`)
    }

    log(`Opening channel to node "${counterpartyStr}"..`)

    const response = await this.api.openChannel(counterpartyStr, amountToFund.toString())

    if (!response.ok) {
      return log(
        await this.failedApiCall(response, `open a channel to ${counterpartyStr}`, {
          400: (v) => `one or more invalid inputs ${v.status}`,
          403: 'not enough balance',
          409: 'channel already exists',
          422: (v) => v.error
        })
      )
    }

    const channelId = await response.json().then((res) => res.channelId)
    return log(`Successfully opened channel "${channelId}" to node "${counterpartyStr}".`)
  }
}
