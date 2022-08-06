import type API from '../utils/api'
import { toPaddedString } from '../utils'
import { Command, type CacheFunctions } from '../utils/command'

export default class Info extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[], 'displays node information']
      },
      api,
      cache
    )
  }

  public name() {
    return 'info'
  }

  public description() {
    return 'Information about the HOPR Node, including any options it started with'
  }

  public async execute(log: (msg: string) => void, _query: string): Promise<void> {
    const nodeInfoRes = await this.api.getInfo()
    if (!nodeInfoRes.ok) return log(this.invalidResponse('get node information'))
    const nodeInfo = await nodeInfoRes.json()

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
