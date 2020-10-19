import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Channel as ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import type { AutoCompleteResult } from './abstractCommand'
import chalk from 'chalk'
import { u8aToHex } from '@hoprnet/hopr-utils'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'
import { AbstractCommand } from './abstractCommand'
import { checkPeerIdInput, styleValue } from '../utils'

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

  async execute(query?: string): Promise<string | void> {
    if (query == null) {
      return styleValue(`Invalid arguments. Expected 'close <peerId>'. Received '${query}'`, 'failure')
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    try {
      const {status, receipt} = await this.node.closeChannel(peerId)

      if (status === 'PENDING') {
        return `${chalk.green(`Closing channel, receipt: ${styleValue(receipt, 'hash')}`)}}.`
      } else {
        return `${chalk.green(`Initiated channel closure, receipt: ${styleValue(receipt, 'hash')}`)}}.`
      }
    } catch (err) {
      return styleValue(err.message, 'failure')
    }
  }

  async autocomplete(query: string = '', line: string = ''): Promise<AutoCompleteResult> {
    if (!query) {
      return [[this.name()], line]
    }

    let peerIdStrings: string[]
    try {
      peerIdStrings = await this.node.paymentChannels.channel.getAll(
        async (channel: ChannelInstance) => (await pubKeyToPeerId(await channel.offChainCounterparty)).toB58String(),
        async (peerIdPromises: Promise<string>[]) => {
          return await Promise.all(peerIdPromises)
        }
      )
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
