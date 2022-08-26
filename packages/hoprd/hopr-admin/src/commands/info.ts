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
    if (!nodeInfoRes.ok) return log(this.failedCommand('get node information'))
    const nodeInfo = await nodeInfoRes.json()

    return log(
      toPaddedString([
        ['Announcing to other nodes as', nodeInfo.announcedAddress.join('  ')],
        ['Listening on', nodeInfo.listeningAddress.join('  ')],
        ['Running on', nodeInfo.network],
        ['Using HOPR environment', nodeInfo.environment],
        ['Channel closure period', `${nodeInfo.channelClosurePeriod} minutes`],
        ['HOPR Token Contract Address', nodeInfo.hoprToken],
        ['HOPR Channels Contract Addresss', nodeInfo.hoprChannels],
        ['HOPR NetworkRegistry Contract Address', nodeInfo.hoprNetworkRegistry],
        ['NetworkRegistry Eligibility', nodeInfo.isEligible],
        ['Connectivity Status', nodeInfo.connectivityStatus]
      ])
    )
  }
}
