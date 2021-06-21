import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { MIN_HOPS_REQUIRED } from '@hoprnet/hopr-core/lib/constants'
import { checkPeerIdInput, encodeMessage, styleValue } from './utils'
import { AbstractCommand, GlobalState } from './abstractCommand'

export class SendMessage extends AbstractCommand {
  constructor(public node: Hopr) {
    super()
  }

  public name() {
    return 'send'
  }

  public help() {
    return 'Sends a message to another party'
  }

  private insertMyAddress(message: string): string {
    const myAddress = this.node.getId().toB58String()
    return `${myAddress}:${message}`
  }

  protected async sendMessage(
    state: GlobalState,
    recipient: PeerId,
    rawMessage: string,
    path?: PeerId[]
  ): Promise<string> {
    const message = state.includeRecipient ? this.insertMyAddress(rawMessage) : rawMessage

    try {
      await this.node.sendMessage(encodeMessage(message), recipient, path)
      return 'Message sent'
    } catch (err) {
      return styleValue(`Could not send message. (${err.message})`, 'failure')
    }
  }

  public async execute(log: (str: string) => void, query: string, state: GlobalState): Promise<void> {
    try {
      let [err, peerIdString, message] = this._assertUsage(query, ['PeerId', 'Message'], /([A-Za-z0-9_,]+)\s(.*)/)
      if (err) throw Error(err)

      if (peerIdString.includes(',')) {
        // Manual routing
        // Direct routing can be done with ,recipient
        const path = await Promise.all(
          peerIdString
            .split(',')
            .filter(Boolean)
            .map((x) => checkPeerIdInput(x, state))
        )
        if (path.length > MIN_HOPS_REQUIRED + 1) {
          throw new Error('Cannot create path longer than MIN_HOPS_REQUIRED')
        }

        const [intermediateNodes, recipient] = [path.slice(0, path.length - 1), path[path.length - 1]]
        console.log(
          `Sending message to ${styleValue(recipient.toB58String(), 'peerId')} via ${path
            .slice(0, path.length - 1)
            .map((current) => styleValue(current.toB58String(), 'peerId'))
            .join(',')} ...`
        )
        log(await this.sendMessage(state, recipient, message, intermediateNodes))

        return
      }

      let peerId = await checkPeerIdInput(peerIdString, state)

      console.log(`Sending message to ${styleValue(peerId.toB58String(), 'peerId')} ...`)
      log(await this.sendMessage(state, peerId, message))
    } catch (err) {
      log(styleValue(err.message, 'failure'))
    }
  }
}
