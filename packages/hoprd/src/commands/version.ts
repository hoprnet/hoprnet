import { AbstractCommand } from './abstractCommand.js'
import type { default as Hopr } from '@hoprnet/hopr-core'

export default class Version extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }
  public name() {
    return 'version'
  }

  public help() {
    return 'Displays the version'
  }

  public async execute(log): Promise<void> {
    log(this.node.getVersion())
  }
}
