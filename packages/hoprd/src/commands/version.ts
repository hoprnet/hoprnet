import { AbstractCommand } from './abstractCommand'
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'

export default class Version extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }
  public name() {
    return 'version'
  }

  public help() {
    return 'Displays the version'
  }

  public async execute(): Promise<string> {
    return this.node.getVersion()
  }
}
