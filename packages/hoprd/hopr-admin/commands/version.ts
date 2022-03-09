import { AbstractCommand } from './abstractCommand'
import { getNodeVer } from '../fetch'

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
    await getNodeVer().then(version => log(version))
  }
}
