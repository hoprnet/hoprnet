import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { startDelayedInterval, u8aToHex, moveDecimalPoint } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import chalk from 'chalk'
import readline from 'readline'
import { checkPeerIdInput, getPeers, getOpenChannels, styleValue } from '../utils'
import { AbstractCommand, AutoCompleteResult } from './abstractCommand'

export abstract class OpenChannelBase extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'open'
  }

  public help() {
    return 'Opens a payment channel between you and the counter party provided'
  }

  protected async validateAmountToFund(amountToFund: BN): Promise<void> {
    const { account } = this.node.paymentChannels
    const myAvailableTokens = await account.balance

    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens)) {
      throw Error(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toString(10)}`)
    }
  }

  public async autocomplete(query: string = '', line: string = ''): Promise<AutoCompleteResult> {
    if (!query) {
      return [[this.name()], line]
    }

    const peersWithOpenChannel = await getOpenChannels(this.node, this.node.getId())
    const allPeers = getPeers(this.node, {
      noBootstrapNodes: true
    })

    const peers = allPeers.reduce((acc: string[], peer: PeerId) => {
      if (!peersWithOpenChannel.find((p: PeerId) => p.id.equals(peer.id))) {
        acc.push(peer.toB58String())
      }
      return acc
    }, [])

    if (peers.length < 1) {
      console.log(styleValue(`\nDoesn't know any new node to open a payment channel with.`, 'failure'))
      return [[''], line]
    }

    const hits = query ? peers.filter((peerId: string) => peerId.startsWith(query)) : peers

    return [hits.length ? hits.map((str: string) => `open ${str}`) : ['open'], line]
  }
}

export class OpenChannel extends OpenChannelBase {
  /**
   * Encapsulates the functionality that is executed once the user decides to open a payment channel
   * with another party.
   * @param query peerId string to send message to
   */
  public async execute(query: string): Promise<string> {
    const [err, counterPartyB58Str, amountToFundStr] = this._assertUsage(query, [
      "counterParty's PeerId",
      'amountToFund'
    ])
    if (err) return styleValue(err, 'failure')

    let counterParty: PeerId
    try {
      counterParty = await checkPeerIdInput(counterPartyB58Str)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    const { types } = this.node.paymentChannels
    const amountToFund = new BN(moveDecimalPoint(amountToFundStr, types.Balance.DECIMALS))

    const unsubscribe = startDelayedInterval(`Submitted transaction. Waiting for confirmation`)
    try {
      const { channelId } = await this.node.openChannel(counterParty, amountToFund)
      unsubscribe()
      return `${chalk.green(`Successfully opened channel`)} ${styleValue(u8aToHex(channelId), 'hash')}`
    } catch (err) {
      unsubscribe()
      return styleValue(err.message, 'failure')
    }
  }
}

export class OpenChannelFancy extends OpenChannelBase {
  constructor(public node: Hopr<HoprCoreConnector>, public rl: readline.Interface) {
    super(node)
  }

  private async selectFundAmount(): Promise<BN> {
    const { types, account } = this.node.paymentChannels
    const myAvailableTokens = await account.balance
    const myAvailableTokensDisplay = moveDecimalPoint(myAvailableTokens.toString(), types.Balance.DECIMALS * -1)

    const tokenQuestion = `How many ${types.Balance.SYMBOL} (${styleValue(`${myAvailableTokensDisplay}`, 'number')} ${
      types.Balance.SYMBOL
    } available) shall get staked? : `

    const amountToFund = await new Promise<string>((resolve) => this.rl.question(tokenQuestion, resolve)).then(
      (input) => {
        return new BN(moveDecimalPoint(input, types.Balance.DECIMALS))
      }
    )

    try {
      await this.validateAmountToFund(amountToFund)
      return amountToFund
    } catch (err) {
      console.log(styleValue(err.message, 'failure'))
      return this.selectFundAmount()
    }
  }

  /**
   * Encapsulates the functionality that is executed once the user decides to open a payment channel
   * with another party.
   * @param query peerId string to send message to
   */
  public async execute(query?: string): Promise<string> {
    if (query == null || query == '') {
      return styleValue(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`, 'failure')
    }

    let counterParty: PeerId
    try {
      counterParty = await checkPeerIdInput(query)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    const amountToFund = await this.selectFundAmount()

    const unsubscribe = startDelayedInterval(`Submitted transaction. Waiting for confirmation`)
    try {
      const { channelId } = await this.node.openChannel(counterParty, amountToFund)
      unsubscribe()
      return `${chalk.green(`Successfully opened channel`)} ${styleValue(u8aToHex(channelId), 'hash')}`
    } catch (err) {
      unsubscribe()
      return styleValue(err.message, 'failure')
    }
  }
}
