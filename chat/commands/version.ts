import type AbstractCommand from './abstractCommand'
import pkg from '../package.json'

export default class Version implements AbstractCommand {
  #display = [`hopr-chat: ${pkg.version}`, `hopr-core: ${pkg.dependencies['@hoprnet/hopr-core']}`].join('\n')

  async execute() {
    console.log(this.#display)
  }

  complete() {}
}
