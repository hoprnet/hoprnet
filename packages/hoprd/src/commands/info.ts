import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand.js'
import type { Multiaddr } from '@multiformats/multiaddr'

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
    const smartContractInfo = this.node.smartContractInfo()
    const channelClosureMins = Math.ceil(smartContractInfo.channelClosureSecs / 60) // convert to minutes

    // @TODO Add connector info etc.
    return log(
      [
        `Announcing to other nodes as: ${(await this.node.getAddressesAnnouncedToDHT()).map((ma) => ma.toString())}`,
        `Listening on: ${this.node.getListeningAddresses().map((ma: Multiaddr) => ma.toString())}`,
        `Running on: ${smartContractInfo.network}`,
        `HOPR Token: ${smartContractInfo.hoprTokenAddress}`,
        `HOPR Channels: ${smartContractInfo.hoprChannelsAddress}`,
        `HOPR NetworkRegistry: ${smartContractInfo.hoprNetworkRegistryAddress}`,
        `Eligibility: ${await this.node.isAllowedAccessToNetwork(this.node.getId())}`,
        `Connectivity Indicator: ${this.node.getConnectivityHealth().toString()}`,
        `Channel closure period: ${channelClosureMins} minutes`
      ].join('\n')
    )
  }
}
