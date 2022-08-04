import type { PeerId } from '@libp2p/interface-peer-id'
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

  async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, , counterparty] = this.assertUsage(query) as [string | undefined, string, PeerId]
    if (error) return log(error)

    log(`Closing channel to "${counterparty}"..`)

    let channelClosurePeriod: any = '?'

    const [closeChannelRes, infoRes] = await Promise.all([
      this.api.closeChannel(counterparty.toString()),
      this.api.getInfo()
    ])

    if (!closeChannelRes.ok) {
      return log('close channel')
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
