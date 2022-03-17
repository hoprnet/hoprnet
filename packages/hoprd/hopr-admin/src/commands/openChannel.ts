import type PeerId from 'peer-id'
import type API from '../utils/api'
import BN from 'bn.js'
import { utils as ethersUtils } from 'ethers'
import { Command } from '../utils/command'

export default class OpenChannel extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [
          [
            ['hoprAddressOrAlias', "counterparty's HOPR address", false],
            ['number', 'Amount of HOPR to fund channel with', false]
          ],
          'opens channel'
        ]
      },
      api,
      extra
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
  public async execute(log, query: string): Promise<void> {
    const [error, , counterparty, amount] = this.assertUsage(query) as [string | undefined, string, PeerId, number]
    if (error) return log(error)

    const amountToFund = new BN(String(ethersUtils.parseEther(String(amount))))
    const myAvailableTokens = await this.api.getBalances().then((d) => new BN(d.hopr))

    if (amountToFund.lten(0)) {
      return log(`Invalid 'amount' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens)) {
      return log(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toString(10)}`)
    }

    log(`Opening channel to node "${counterparty.toB58String()}"..`)

    try {
      const response = await this.api.openChannel(counterparty.toB58String(), amountToFund.toString())
      if (response.status == 201) {
        const channelId = response.json().then((res) => res.channelId)
        return log(`Successfully opened channel "${channelId}" to node "${counterparty.toB58String()}".`)
      } else {
        const status = response.json().then((res) => res.status)
        return log(status)
      }
    } catch (error) {
      return log(error.message)
    }
  }
}
