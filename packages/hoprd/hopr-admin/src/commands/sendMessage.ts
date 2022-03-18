import type PeerId from 'peer-id'
import type API from '../utils/api'
import { Command, CMD_PARAMS } from '../utils/command'

export default class SendMessage extends Command {
  constructor(api: API, extra: { getCachedAliases: () => Record<string, string> }) {
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
      extra
    )
  }

  public name() {
    return 'send'
  }

  public description() {
    return 'Sends a message to another party'
  }

  private async insertMyAddress(message: string): Promise<string> {
    const myAddress: string = await this.api.getAddresses().then((res) => res.hopr)
    return `${myAddress}:${message}`
  }

  private async sendMessage(recipient: string, rawMessage: string, path?: string[]): Promise<string> {
    const includeRecipient = await this.api.getSettings().then((res) => res.includeRecipient)
    const message = includeRecipient ? await this.insertMyAddress(rawMessage) : rawMessage

    try {
      const response = await this.api.sendMessage(message, recipient, path)
      if (response.status === 204) {
        return 'Message sent'
      } else {
        return `Could not send message. ${response.status}`
      }
    } catch (err) {
      return `Could not send message.`
    }
  }

  public async execute(log, query: string): Promise<void> {
    const [error, use, pathOrRecipeint, message] = this.assertUsage(query) as [string | undefined, string, any, string]
    if (error) return log(error)

    if (use === 'manual') {
      const pathStr: string = pathOrRecipeint

      if (pathStr.includes(',')) {
        // Direct routing can be done with ,recipient
        const peerIdStrings = pathStr.split(',').filter(Boolean)

        const validatePeerIdOrAlias = CMD_PARAMS.hoprAddressOrAlias[1]
        const aliases = this.extra.getCachedAliases()

        const path: string[] = []
        for (const pIdString of peerIdStrings) {
          try {
            const [valid, peerId] = validatePeerIdOrAlias(pIdString, { aliases })
            if (!valid) throw Error()
            path.push(peerId.toB58String())
          } catch (err) {
            return log(`<${pIdString}> is neither a valid alias nor a valid Hopr address string`)
          }
        }

        const [intermediateNodes, recipient] = [path.slice(0, path.length - 1), path[path.length - 1]]
        console.log(
          `Sending message to ${recipient.toString()} via ${path
            .slice(0, path.length - 1)
            .map((current) => current.toString())
            .join(',')} ...`
        )

        return log(await this.sendMessage(recipient, message, intermediateNodes))
      } else {
        return log(this.invalidUsage(query))
      }
    } else {
      const receiver: PeerId = pathOrRecipeint
      console.log(`Sending message to ${receiver.toB58String()} ..`)
      log(await this.sendMessage(receiver.toB58String(), message))
    }
  }
}
