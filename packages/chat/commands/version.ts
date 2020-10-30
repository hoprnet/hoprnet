import {AbstractCommand} from './abstractCommand'
import pkg from '../package.json'
import {styleValue, getPaddingLength} from '../utils'

export default class Version extends AbstractCommand {
  private items: {
    name: string
    version: string
  }[] = [
    {
      name: 'hopr-chat',
      version: pkg.version
    },
    {
      name: 'hopr-core',
      version: pkg.dependencies['@hoprnet/hopr-core']
    }
  ]

  public name() {
    return 'version'
  }

  public help() {
    return 'Displays the versions for `hopr-chat` and `hopr-core`'
  }

  public async execute(): Promise<string> {
    const items = this.items.map((item) => ({
      name: item.name + ': ',
      version: styleValue(item.version, 'number')
    }))
    const paddingLength = getPaddingLength(
      items.map((item) => item.name),
      false
    )

    return items
      .map((item) => {
        return `${item.name.padEnd(paddingLength)}${item.version}`
      })
      .join('\n')
  }
}
