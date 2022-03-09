import { AbstractCommand } from './abstractCommand'
import { getNodeInfo } from '../fetch'

export class Info extends AbstractCommand {
  constructor() {
    super()
  }

  public name() {
    return 'info'
  }

  public help() {
    return 'Information about the HOPR Node, including any options it started with'
  }

  public async execute(log): Promise<void> {
    const nodeInfo = await getNodeInfo()

    // @TODO Add connector info etc.
    return log(
      [
        `Announcing to other nodes as: ${(nodeInfo.announcedAddress).map((ma) => ma.toString())}`,
        `Listening on: ${nodeInfo.listeningAddress.map((ma) => ma.toString())}`,
        `Running on: ${nodeInfo.network}`,
        `HOPR Token: ${nodeInfo.hoprToken}`,
        `HOPR Channels: ${nodeInfo.hoprChannels}`,
        `Channel closure period: ${nodeInfo.channelClosurePeriod} minutes`
      ].join('\n')
    )
  }
}
