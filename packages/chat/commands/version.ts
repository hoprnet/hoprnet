import { AbstractCommand } from './abstractCommand'
import pkg from '../package.json'

export default class Version extends AbstractCommand {
  #display = [`hopr-chat: ${pkg.version}`, `hopr-core: ${pkg.dependencies['@hoprnet/hopr-core']}`].join('\n')
  name() {
    return 'version'
  }
  help() {
    return 'shows the versions for `hopr-chat` and `hopr-core`'
  }

  async execute(): Promise<string> {
    return this.#display
  }
}
