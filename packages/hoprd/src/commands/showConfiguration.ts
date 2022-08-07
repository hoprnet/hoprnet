import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand.js'

export class ShowConfiguration extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'showConfiguration'
  }

  public help() {
    return 'Configuration of the HOPR Node'
  }

  public async execute(log): Promise<void> {
    const config = this.node.getPublicHoprOptions()
    const { hoprTokenAddress, hoprChannelsAddress, channelClosureSecs, hoprNetworkRegistryAddress } =
      this.node.smartContractInfo()
    const channelClosureMins = Math.ceil(channelClosureSecs / 60) // convert to minutes
    return log(
      [
        `Environment: ${config.environment}`,
        `Network: ${config.network}`,
        `HOPR Token: ${hoprTokenAddress}`,
        `HOPR Channels: ${hoprChannelsAddress}`,
        `HOPR NetworkRegistry: ${hoprNetworkRegistryAddress}`,
        `Eligibility: ${await this.node.isAllowedAccessToNetwork(this.node.getId())}`,
        `Channel closure period: ${channelClosureMins} minutes`
      ].join('\n')
    )
  }
}
