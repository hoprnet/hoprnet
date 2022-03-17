import type PeerId from 'peer-id'
import type API from '../utils/api'
import { Command } from '../utils/command'

export default class CloseChannel extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[['hoprAddressOrAlias', "counterparty's HOPR address", false]], 'closes channel']
      },
      api,
      extra
    )
  }

  public name() {
    return 'close'
  }

  public description() {
    return 'Close an open channel'
  }

  async execute(log, query: string): Promise<void> {
    const [error, , counterparty] = this.assertUsage(query) as [string | undefined, string, PeerId]
    if (error) return log(error)

    log(`Closing channel to "${counterparty}"..`)

    try {
      const [response, info] = await Promise.all([
        this.api.closeChannel(counterparty.toB58String()),
        this.api.getInfo()
      ])

      if (response.status === 200) {
        const { receipt, channelStatus } = await response.json()

        if (channelStatus === 'Open') {
          return log(
            `Initiated channel closure, the channel must remain open for at least ${info.channelClosurePeriod} minutes. Please send the close command again once the cool-off has passed. Receipt: "${receipt}".`
          )
        } else {
          return log(`Closing channel. Receipt: ${receipt}`)
        }
      } else {
        const { status, error } = (await response.json()) as any
        return log(`Unable to close channel with status code "${response.status}". ${status} ${error}`)
      }
    } catch (error) {
      return log(error.message)
    }
  }
}
