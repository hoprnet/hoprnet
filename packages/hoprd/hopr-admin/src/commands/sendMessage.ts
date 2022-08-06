import type API from '../utils/api'
import { Command, CMD_PARAMS, type CacheFunctions } from '../utils/command'

export default class SendMessage extends Command {
  constructor(api: API, cache: CacheFunctions) {
    super(
      {
        default: [
          [
            ['hoprAddressOrAlias', 'recipient of the message', false],
            ['string', 'message to send', false]
          ],
          'send a message, path is chosen automatically'
        ],
        manual: [
          [
            ['string', "the path of the message seperated by ',', last one is the receiver", false],
            ['string', 'message to send', false]
          ],
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

    const validatePeerIdOrAlias = CMD_PARAMS.hoprAddressOrAlias[1]
    const aliases = this.cache.getCachedAliases()

    const path: string[] = []
    for (const pIdString of peerIdStrings) {
      try {
        const [valid, peerId] = validatePeerIdOrAlias(pIdString, { aliases })
        if (!valid) throw Error()
        path.push(peerId.toB58String())
      } catch (err) {
        return [false, `<${pIdString}> is neither a valid alias nor a valid Hopr address string`, undefined]
      }
    }

    const [intermediateNodes, recipient] = [path.slice(0, path.length - 1), path[path.length - 1]]
    return [true, undefined, { intermediateNodes, recipient }]
  }

  public async execute(log: (msg: string) => void, query: string): Promise<void> {
    const [error, use, pathOrRecipeint, message] = this.assertUsage(query) as [string | undefined, string, any, string]
    if (error) return log(error)

    let path: string[] | undefined
    let recipient: string = pathOrRecipeint

    if (use === 'manual') {
      const [valid, error, result] = this.parsePathStr(pathOrRecipeint)
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

    log(`Sending message to ${recipient} ..`)

    const payload = settings.includeRecipient ? `${addresses.hopr}:${message}` : message
    const response = await this.api.sendMessage(payload, recipient, path)

    if (!response.ok) {
      return log(this.invalidResponse('send message'))
    } else {
      return log(`Message to ${recipient} sent`)
    }
  }
}
