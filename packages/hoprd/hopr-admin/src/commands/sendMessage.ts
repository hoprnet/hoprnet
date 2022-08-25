import type API from '../utils/api'
import { Command, CMD_PARAMS, type CacheFunctions } from '../utils/command'

export default class SendMessage extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [[['hoprAddressOrAlias'], ['string', 'message']], 'send a message, path is chosen automatically'],
        manual: [
          [['hoprAddressOrAlias'], ['string', "path seperated by ','"], ['string', 'message']],
          'send a message, path is manually specified'
        ]
      },
      api,
      cache
    )
  }

  public name() {
    return 'send'
  }

  public description() {
    return 'Sends a message to another party'
  }

  private parsePathStr(
    pathStr: string
  ): [valid: boolean, error: string | undefined, path?: { intermediateNodes: string[]; recipient: string }] {
    const peerIdStrings = pathStr.split(',').filter(Boolean)

    const validatePeerIdOrAlias = CMD_PARAMS.hoprAddressOrAlias[2]
    const aliases = this.cache.getCachedAliases()

    const path: string[] = []
    for (const pIdString of peerIdStrings) {
      try {
        const [valid, peerId] = validatePeerIdOrAlias(pIdString, { aliases })
        if (!valid) throw Error()
        path.push(peerId.toString())
      } catch (err) {
        return [false, `<${pIdString}> is neither a valid alias nor a valid Hopr address string`, undefined]
      }
    }

    const [intermediateNodes, recipient] = [path.slice(0, path.length - 1), path[path.length - 1]]
    return [true, undefined, { intermediateNodes, recipient }]
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, use, pathOrRecipient, message] = this.assertUsage(query) as [string | undefined, string, any, string]
    if (error) return log(error)

    let path: string[] | undefined
    let recipient: string = pathOrRecipient

    if (use === 'manual') {
      const [valid, error, result] = this.parsePathStr(pathOrRecipient)
      if (!valid) return log(`${error}\n${this.usage()}`)
      path = result.intermediateNodes
      recipient = result.recipient
    }

    const [settingsRes, addressesRes] = await Promise.all([this.api.getSettings(), this.api.getAddresses()])
    if (!settingsRes.ok || !addressesRes.ok) {
      return log(this.invalidResponse('send message'))
    }
    const settings = await settingsRes.json()
    const addresses = await addressesRes.json()

    if (path && path.length == 0) {
      log(`Sending message to ${recipient} via ${path.join('->')} ...`)
    } else if (path && path.length > 0) {
      log(`Sending direct message to ${recipient} ...`)
    } else {
      log(`Sending message to ${recipient} using automatic path finding ...`)
    }

    const payload = settings.includeRecipient ? `${addresses.hopr}:${message}` : message
    const response = await this.api.sendMessage(payload, recipient, path)

    if (!response.ok) {
      return log(this.invalidResponse('send message'))
    } else {
      return log(`Message to ${recipient} sent`)
    }
  }
}
