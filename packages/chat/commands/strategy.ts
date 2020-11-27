import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import { AbstractCommand } from './abstractCommand'

export class SetStrategy extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'strategy'
  }

  public help() {
    return 'set an automatic strategy for the node.'
  }

  async execute(query: string): Promise<string> {
    try {
      this.node.setChannelStrategy(query as any) 
      return "Strategy was set"
    } catch {
      return "Could not set strategy. Try PASSIVE or PROMISCUOUS"
    }
  }
}
