import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { startDelayedInterval, u8aToHex, moveDecimalPoint } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import chalk from 'chalk'
import readline from 'readline'
import { checkPeerIdInput, getPeers, getOpenChannels } from '../utils'
import { AbstractCommand, AutoCompleteResult } from './abstractCommand'

export abstract class OpenChannelBase extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super()
  }

  public name() {
    return 'open'
  }

  public help() {
    return 'opens a payment channel'
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

  /**
   * Open's a payment channel
   *
   * @param counterParty the counter party's peerId
   * @param amountToFund the amount to fund in HOPR(wei)
   */
  protected async openChannel(
    counterParty: PeerId,
    amountToFund: BN
  ): Promise<{
    channelId: Types.Hash
  }> {
    const { utils, types, account } = this.node.paymentChannels
    const self = this.node.peerInfo.id

    const channelId = await utils.getId(
      await utils.pubKeyToAccountId(self.pubKey.marshal()),
      await utils.pubKeyToAccountId(counterParty.pubKey.marshal())
    )

    const myAvailableTokens = await account.balance

    // validate 'amountToFund'
    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens)) {
      throw Error(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toString(10)}`)
    }

    const amPartyA = utils.isPartyA(
      await utils.pubKeyToAccountId(self.pubKey.marshal()),
      await utils.pubKeyToAccountId(counterParty.pubKey.marshal())
    )

    const channelBalance = types.ChannelBalance.create(
      undefined,
      amPartyA
        ? {
            balance: amountToFund,
            balance_a: amountToFund,
          }
        : {
            balance: amountToFund,
            balance_a: new BN(0),
          }
    )

    await this.node.paymentChannels.channel.create(
      counterParty.pubKey.marshal(),
      async () => this.node.interactions.payments.onChainKey.interact(counterParty),
      channelBalance,
      (balance: Types.ChannelBalance): Promise<Types.SignedChannel> =>
        this.node.interactions.payments.open.interact(counterParty, balance)
    )

    return {
      channelId,
    }
  }

  public async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
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
      'amountToFund',
    ])

    if (err) {
      throw new Error(err)
    }

    let counterParty: PeerId
    try {
      counterParty = await checkPeerIdInput(counterPartyB58Str)
    } catch (err) {
      return chalk.red(err.message)
    }

    const amountToFund = new BN(amountToFundStr)

    const unsubscribe = startDelayedInterval(`Submitted transaction. Waiting for confirmation`)
    try {
      const { channelId } = await this.openChannel(counterParty, amountToFund)
      unsubscribe()
      return `${chalk.green(`Successfully opened channel`)} ${chalk.yellow(u8aToHex(channelId))}`
    } catch (err) {
      unsubscribe()
      return chalk.red(err.message)
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

    const tokenQuestion = `How many ${types.Balance.SYMBOL} (${chalk.magenta(
      `${myAvailableTokensDisplay} ${types.Balance.SYMBOL}`
    )} available) shall get staked? : `

    const amountToFund = await new Promise<string>((resolve) => this.rl.question(tokenQuestion, resolve)).then(
      (input) => {
        return new BN(moveDecimalPoint(input, types.Balance.DECIMALS))
      }
    )

    try {
      await this.validateAmountToFund(amountToFund)
      return amountToFund
    } catch (err) {
      console.log(chalk.red(err.message))
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
      return chalk.red(`Invalid arguments. Expected 'open <peerId>'. Received '${query}'`)
    }

    let counterParty: PeerId
    try {
      counterParty = await checkPeerIdInput(query)
    } catch (err) {
      return chalk.red(err.message)
    }

    const amountToFund = await this.selectFundAmount()

    const unsubscribe = startDelayedInterval(`Submitted transaction. Waiting for confirmation`)
    try {
      const { channelId } = await this.openChannel(counterParty, amountToFund)
      unsubscribe()
      return `${chalk.green(`Successfully opened channel`)} ${chalk.yellow(u8aToHex(channelId))}`
    } catch (err) {
      unsubscribe()
      return chalk.red(err.message)
    }
  }
}
