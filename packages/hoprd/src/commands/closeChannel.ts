import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import type { AutoCompleteResult } from './abstractCommand'
import chalk from 'chalk'
import { pubKeyToPeerId } from '@hoprnet/hopr-utils'
import { AbstractCommand, GlobalState } from './abstractCommand'
import { checkPeerIdInput, styleValue } from './utils'

export default class CloseChannel extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'close'
  }

  public help() {
    return 'Close an open channel'
  }

  async execute(query: string, state: GlobalState): Promise<string | void> {
    if (query == null) {
      return styleValue(`Invalid arguments. Expected 'close <peerId>'. Received '${query}'`, 'failure')
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query, state)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    try {
      const { status, receipt } = await this.node.closeChannel(peerId)

      if (status === 'PENDING') {
        return `${chalk.green(`Closing channel. Receipt: ${styleValue(receipt, 'hash')}`)}.`
      } else {
        return `${chalk.green(
          `Initiated channel closure, the channel must remain open for at least 2 minutes. Please send the close command again once the cool-off has passed. Receipt: ${styleValue(
            receipt,
            'hash'
          )}`
        )}.`
      }
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }

  async autocomplete(query: string = '', line: string = ''): Promise<AutoCompleteResult> {
    const ethereum = this.node.paymentChannels
    const selfPubKey = new ethereum.types.Public(this.node.getId().pubKey.marshal())
    const self = await selfPubKey.toAddress()

    // get channels which are ours & open
    const channels = await ethereum.indexer.getChannels(async (channel) => {
      // must be one of ours
      if (!self.eq(channel.partyA) && !self.eq(channel.partyB)) return false
      // must be open
      if (channel.getStatus() !== 'CLOSED') return false

      return true
    })

    let peerIdStrings: string[]
    try {
      for (const channel of channels) {
        const counterparty = channel.partyA.eq(self) ? channel.partyB : channel.partyA
        const pubKey = await ethereum.indexer.getPublicKeyOf(counterparty)
        const peerId = await pubKeyToPeerId(pubKey)
        peerIdStrings.push(peerId.toB58String())
      }
    } catch (err) {
      console.log(styleValue(err.message), 'failure')
      return [[], line]
    }

    if (peerIdStrings != null && peerIdStrings.length < 1) {
      console.log(styleValue(`\nCannot find any open channels to close.`), 'failure')
      return [[''], line]
    }

    const hits = query ? peerIdStrings.filter((peerId: string) => peerId.startsWith(query)) : peerIdStrings
    return [hits.length ? hits.map((str: string) => `close ${str}`) : ['close'], line]
  }
}
