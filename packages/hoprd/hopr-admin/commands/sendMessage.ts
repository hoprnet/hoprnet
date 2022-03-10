import type PeerId from 'peer-id'
import { checkPeerIdInput, encodeMessage, styleValue } from './utils'
import { AbstractCommand } from './abstractCommand'
import { getAddresses, getSettings, sendMessage } from '../fetch'

export class SendMessage extends AbstractCommand {
  constructor() {
    super()
  }

  public name() {
    return 'send'
  }

  public help() {
    return 'Sends a message to another party'
  }

  private static async insertMyAddress(message: string): string {
    // TODO: .toB58String() needed???
    const myAddress = await getAddresses().then(res => res.hoprAddress)
    return `${myAddress}:${message}`
  }

  protected async sendMessage(
    recipient: PeerId,
    rawMessage: string,
    path?: PublicKey[]
  ): Promise<string> {
    const includeRecipientValue = await getSettings().then(res => res.includeRecipient)
    const message = includeRecipientValue ? SendMessage.insertMyAddress(rawMessage) : rawMessage

    try {
      await sendMessage(encodeMessage(message), recipient, path)
      return 'Message sent'
    } catch (err) {
      return styleValue(`Could not send message. (${err.message})`, 'failure')
    }
  }

  public async execute(log: (str: string) => void, query: string): Promise<void> {
    let [err, peerIdString, message] = this._assertUsage(query, ['PeerId', 'Message'], /([A-Za-z0-9_,]+)\s(.*)/)
    if (err) {
      log(styleValue(err, 'failure'))
      return
    }

    if (peerIdString.includes(',')) {
      // Manual routing
      // Direct routing can be done with ,recipient
      const peerIdStrings = peerIdString.split(',').filter(Boolean)

      const path: PublicKey[] = []
      for (const pIdString of peerIdStrings) {
        try {
          path.push(PublicKey.fromPeerId(checkPeerIdInput(pIdString)))
        } catch (err) {
          log(styleValue(`<${pIdString}> is neither a valid alias nor a valid Hopr address string`))
          return
        }
      }

      if (path.length > INTERMEDIATE_HOPS + 1) {
        log(styleValue('Cannot create path longer than INTERMEDIATE_HOPS', 'failure'))
        return
      }

      const [intermediateNodes, recipient] = [path.slice(0, path.length - 1), path[path.length - 1]]
      console.log(
        `Sending message to ${styleValue(recipient.toString(), 'peerId')} via ${path
          .slice(0, path.length - 1)
          .map((current) => styleValue(current.toString(), 'peerId'))
          .join(',')} ...`
      )
      log(await this.sendMessage(recipient.toPeerId(), message, intermediateNodes))

      return
    }

    let destination: PeerId
    try {
      destination = checkPeerIdInput(peerIdString)
    } catch (err) {
      log(styleValue(`<${peerIdString}> is neither a valid alias nor a valid Hopr address string`))
      return
    }

    console.log(`Sending message to ${styleValue(destination.toB58String(), 'peerId')} ...`)
    log(await this.sendMessage(destination, message))
  }
}
