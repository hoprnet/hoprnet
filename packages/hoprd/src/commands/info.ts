import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand.js'

export class Info extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'info'
  }

  public help() {
    return 'Information about the HOPR Node, including any options it started with'
  }

  public async execute(log): Promise<void> {
    const smartContractInfo = await this.node.smartContractInfo()
    const channelClosureMins = Math.ceil(smartContractInfo.channelClosureSecs / 60) // convert to minutes

    // @TODO Add connector info etc.
    return log(
      [
        `Announcing to other nodes as: ${(await this.node.getAnnouncedAddresses()).map((ma) => ma.toString())}`,
        `Listening on: ${this.node.getListeningAddresses().map((ma) => ma.toString())}`,
        `Running on: ${smartContractInfo.network}`,
        `HOPR Token: ${smartContractInfo.hoprTokenAddress}`,
        `HOPR Channels: ${smartContractInfo.hoprChannelsAddress}`,
        `Channel closure period: ${channelClosureMins} minutes`
      ].join('\n')
    )
  }
}
