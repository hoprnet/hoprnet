import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Channel as ChannelInstance } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'

import BN from 'bn.js'
import chalk from 'chalk'

import type { AutoCompleteResult } from './abstractCommand'
import { AbstractCommand } from './abstractCommand'
import { checkPeerIdInput } from '../utils'
import { startDelayedInterval, u8aToHex } from '@hoprnet/hopr-utils'
import { pubKeyToPeerId } from '@hoprnet/hopr-core/lib/utils'

export default class CloseChannel extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  name() {
    return 'close'
  }

  help() {
    return 'Close a channel' //TODO 
  }

  async execute(query?: string): Promise<any> {
    if (query == null) {
      console.log(chalk.red(`Invalid arguments. Expected 'close <peerId>'. Received '${query}'`))
      return
    }

    let peerId: PeerId
    try {
      peerId = await checkPeerIdInput(query)
    } catch (err) {
      console.log(err.message)
      return
    }

    const unsubscribe = startDelayedInterval(`Initiated settlement. Waiting for finalisation`)

    let channel: ChannelInstance

    try {
      channel = await this.node.paymentChannels.channel.create(
        peerId.pubKey.marshal(),
        async (counterparty: Uint8Array) =>
          this.node.interactions.payments.onChainKey.interact(await pubKeyToPeerId(counterparty))
      )

      await channel.initiateSettlement()

      console.log(
        `${chalk.green(`Successfully closed channel`)} ${chalk.yellow(
          u8aToHex(await channel.channelId)
        )}. Received ${chalk.magenta(new BN(0).toString())} ${this.node.paymentChannels.types.Balance.SYMBOL}.`
      )
    } catch (err) {
      console.log(chalk.red(err.message))
    }

    await new Promise((resolve) =>
      setTimeout(() => {
        unsubscribe()
        process.stdout.write('\n')
        resolve()
      })
    )
  }

  async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    let peerIdStrings: string[]
    try {
      peerIdStrings = await this.node.paymentChannels.channel.getAll(
        async (channel: ChannelInstance) => (await pubKeyToPeerId(await channel.offChainCounterparty)).toB58String(),
        async (peerIdPromises: Promise<string>[]) => {
          return await Promise.all(peerIdPromises)
        }
      )
    } catch (err) {
      console.log(chalk.red(err.message))
      return [[], line]
    }

    if (peerIdStrings != null && peerIdStrings.length < 1) {
      console.log(
        chalk.red(
          `\nCannot close any channel because there are not any open ones and/or channels were opened by a third party.`
        )
      )
      return [[''], line]
    }

    const hits = query ? peerIdStrings.filter((peerId: string) => peerId.startsWith(query)) : peerIdStrings
    return [hits.length ? hits.map((str: string) => `close ${str}`) : ['close'], line]
  }
}
