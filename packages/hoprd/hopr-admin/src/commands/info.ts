import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command } from '../utils/command'

export default class Info extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
    super(
      {
        default: [[], 'displays node information']
      },
      api,
      extra
    )
  }

  public name() {
    return 'info'
  }

  public description() {
    return 'Information about the HOPR Node, including any options it started with'
  }

  public async execute(log): Promise<void> {
    const nodeInfo = await this.api.getInfo()

    // TODO: Add connector info etc.
    return log(
      toPaddedString([
        ['Announcing to other nodes as', nodeInfo.announcedAddress.join('\n')],
        ['Listening on', nodeInfo.listeningAddress.join('\n')],
        ['Running on', nodeInfo.network],
        ['HOPR Token', nodeInfo.hoprToken],
        ['HOPR Channels', nodeInfo.hoprChannels],
        ['Channel closure period', `${nodeInfo.channelClosurePeriod} minutes`]
      ])
    )
  }
}
