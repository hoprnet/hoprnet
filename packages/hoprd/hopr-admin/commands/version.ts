import { AbstractCommand } from './abstractCommand'

export default class Version extends AbstractCommand {
  constructor() {
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
