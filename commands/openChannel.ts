import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type AbstractCommand from './abstractCommand'

import BigNumber from 'bignumber.js'
import BN from 'bn.js'

import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'

import chalk from 'chalk'

import { checkPeerIdInput, getPeers, getOpenChannels } from '../utils'
import { clearString, startDelayedInterval, u8aToHex } from '@hoprnet/hopr-utils'
import readline from 'readline'

export default class OpenChannel implements AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {}

  /**
   * Encapsulates the functionality that is executed once the user decides to open a payment channel
   * with another party.
   * @param query peerId string to send message to
   */
  async execute(rl: readline.Interface, query?: string): Promise<void> {
    if (query == null || query == '') {
      console.log(chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`))
      return
    }

    let counterparty: PeerId
    try {
      counterparty = await checkPeerIdInput(query)
    } catch (err) {
      console.log(err.message)
      return
    }

    const channelId = await this.node.paymentChannels.utils.getId(
      /* prettier-ignore */
      await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()),
      await this.node.paymentChannels.utils.pubKeyToAccountId(counterparty.pubKey.marshal())
    )

    const tokens = new BigNumber((await this.node.paymentChannels.account.balance).toString()).div(
      new BigNumber(10).pow(this.node.paymentChannels.types.Balance.DECIMALS)
    )
    let funds: BigNumber, tmpFunds: string
    const tokenQuestion = `How many ${this.node.paymentChannels.types.Balance.SYMBOL} (${chalk.magenta(
      `${tokens.toString()} ${this.node.paymentChannels.types.Balance.SYMBOL}`
    )} available) shall get staked? : `
    const exitQuestion = `Do you want to cancel (${chalk.green('Y')} / ${chalk.red('n')}) : `

    do {
      tmpFunds = await new Promise<string>((resolve) => rl.question(tokenQuestion, resolve))
      try {
        funds = new BigNumber(tmpFunds)
      } catch {}
      clearString(tokenQuestion + tmpFunds, rl)

      if (tmpFunds.length == 0) {
        let decision = await new Promise<string>((resolve) => rl.question(exitQuestion, resolve))
        if (decision.length == 0 || decision.match(/^y(es)?$/i)) {
          clearString(exitQuestion + decision, rl)

          return
        }
        clearString(exitQuestion + decision, rl)
      }
    } while (funds == null || funds.lte(0) || funds.gt(tokens) || funds.isNaN())

    const channelFunding = new BN(
      funds.times(new BigNumber(10).pow(this.node.paymentChannels.types.Balance.DECIMALS)).toString()
    )

    const isPartyA = this.node.paymentChannels.utils.isPartyA(
      await this.node.paymentChannels.utils.pubKeyToAccountId(this.node.peerInfo.id.pubKey.marshal()),
      await this.node.paymentChannels.utils.pubKeyToAccountId(counterparty.pubKey.marshal())
    )

    const channelBalance = this.node.paymentChannels.types.ChannelBalance.create(
      undefined,
      isPartyA
        ? {
            balance: channelFunding,
            balance_a: channelFunding,
          }
        : {
            balance: channelFunding,
            balance_a: new BN(0),
          }
    )

    const unsubscribe = startDelayedInterval(`Submitted transaction. Waiting for confirmation`)

    try {
      await this.node.paymentChannels.channel.create(
        counterparty.pubKey.marshal(),
        async () =>
          this.node.paymentChannels.utils.pubKeyToAccountId(
            await this.node.interactions.payments.onChainKey.interact(counterparty)
          ),
        channelBalance,
        (balance: Types.ChannelBalance): Promise<Types.SignedChannel> =>
          this.node.interactions.payments.open.interact(counterparty, balance)
      )

      console.log(`${chalk.green(`Successfully opened channel`)} ${chalk.yellow(u8aToHex(channelId))}`)
    } catch (err) {
      console.log(chalk.red(err.message))
    }

    unsubscribe()
  }

  async complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void, query?: string) {
    const peersWithOpenChannel = await getOpenChannels(this.node, this.node.peerInfo.id)
    const allPeers = getPeers(this.node, {
      noBootstrapNodes: true,
    })

    const peers = allPeers.reduce((acc: string[], peer: PeerId) => {
      if (!peersWithOpenChannel.find((p: PeerId) => p.id.equals(peer.id))) {
        acc.push(peer.toB58String())
      }
      return acc
    }, [])

    if (peers.length < 1) {
      console.log(chalk.red(`\nDoesn't know any new node to open a payment channel with.`))
      return cb(undefined, [[''], line])
    }

    const hits = query ? peers.filter((peerId: string) => peerId.startsWith(query)) : peers

    return cb(undefined, [hits.length ? hits.map((str: string) => `open ${str}`) : ['open'], line])
  }
}
