import type { PeerId } from '@libp2p/interface-peer-id'
import type API from '../utils/api'
import { Command, type CacheFunctions, type ChannelDirection } from '../utils/command'

export default class CloseChannel extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[['hoprAddressOrAlias'], ['direction']], 'closes channel']
      },
      api,
      cache
    )
  }

  public name() {
    return 'close'
  }

  public description() {
    return 'Close an an outgoing channel'
  }

  async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , counterparty, direction] = this.assertUsage(query) as [
      string | undefined,
      string,
      PeerId,
      ChannelDirection
    ]
    if (error) return log(error)

    log(`Closing channel to "${counterparty}"..`)

    let channelClosurePeriod: any = '?'

    const [closeChannelRes, infoRes] = await Promise.all([
      this.api.closeChannel(counterparty.toString(), direction),
      this.api.getInfo()
    ])

    if (!closeChannelRes.ok) {
      return log(
        await this.failedApiCall(closeChannelRes, `close channel with '${counterparty.toString()}'`, {
          400: `invalid peer ID ${counterparty.toString()}`,
          422: (v) => v.error
        })
      )
    }

    if (!infoRes.ok) {
      return log(
        await this.failedApiCall(
          infoRes,
          `close channel with '${counterparty.toString()}' when fetching node information`,
          {
            422: (v) => v.error
          }
        )
      )
    }

    if (infoRes.ok) {
      channelClosurePeriod = (await infoRes.json()).channelClosurePeriod
    }

    const { receipt, channelStatus } = await closeChannelRes.json()

    if (channelStatus === 'Open') {
      return log(
        `Initiated channel closure, the channel must remain open for at least ${channelClosurePeriod} minutes. Please send the close command again once the cool-off has passed. Receipt: "${receipt}".`
      )
    } else {
      if (receipt != undefined) return log(`Closed channel. Receipt: ${receipt}`)
      else return log(`Closing channel, closure window still active.`)
    }
  }
}
