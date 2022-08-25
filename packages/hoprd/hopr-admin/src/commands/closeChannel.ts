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
      log(`Channel to "${counterparty}" closed.`)
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
      return log(`Closing channel. Receipt: ${receipt}`)
    }
  }
}
