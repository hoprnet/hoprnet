import { AbstractCommand } from './abstractCommand'
import pkg from '../package.json'

export default class Version extends AbstractCommand {
  private display = [`hopr-chat: ${pkg.version}`, `hopr-core: ${pkg.dependencies['@hoprnet/hopr-core']}`].join('\n')

  public name() {
    return 'version'
  }

  public help() {
    return 'shows the versions for `hopr-chat` and `hopr-core`'
  }

  public async execute(): Promise<string> {
    return this.display
  }
}
