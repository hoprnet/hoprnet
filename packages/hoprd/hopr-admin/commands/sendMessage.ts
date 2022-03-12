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
    const myAddress: string = await getAddresses().then(res => res.hoprAddress)
    return `${myAddress}:${message}`
  }

  protected async sendMessage(
    recipient: string,
    rawMessage: string,
    path?: string[]
  ): Promise<string> {
    const includeRecipientValue = await getSettings().then(res => res.includeRecipient)
    const message = includeRecipientValue ? SendMessage.insertMyAddress(rawMessage) : rawMessage

    try {
      const response = await sendMessage(message, recipient, path)
      const { status, error } = await response.json()
      if (response.status === 204) {
        return 'Message sent'
      } else {
        return styleValue(`Could not send message. (${error})`, 'failure')
      }
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

      const path: string[] = []
      for (const pIdString of peerIdStrings) {
        try {
          path.push(await checkPeerIdInput(pIdString).then(peerId => peerId.toB58String()))
        } catch (err) {
          log(styleValue(`<${pIdString}> is neither a valid alias nor a valid Hopr address string`))
          return
        }
      }

      const [intermediateNodes, recipient] = [path.slice(0, path.length - 1), path[path.length - 1]]
      console.log(
        `Sending message to ${styleValue(recipient.toString(), 'peerId')} via ${path
          .slice(0, path.length - 1)
          .map((current) => styleValue(current.toString(), 'peerId'))
          .join(',')} ...`
      )
      log(await this.sendMessage(recipient, message, intermediateNodes))

      return
    }

    let destination: PeerId
    try {
      destination = await checkPeerIdInput(peerIdString)
    } catch (err) {
      log(styleValue(`<${peerIdString}> is neither a valid alias nor a valid Hopr address string`))
      return
    }

    console.log(`Sending message to ${styleValue(destination.toB58String(), 'peerId')} ...`)
    log(await this.sendMessage(destination.toB58String(), message))
  }
}
