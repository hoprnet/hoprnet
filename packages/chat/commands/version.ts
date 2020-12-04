import { AbstractCommand } from './abstractCommand'
import { FULL_VERSION } from '@hoprnet/hopr-core'

export default class Version extends AbstractCommand {
  public name() {
    return 'version'
  }

  public help() {
    return 'Displays the version'
  }

  public async execute(): Promise<string> {
    return FULL_VERSION
  }
}
