import type Hopr from '@hoprnet/hopr-core'
import type PeerId from 'peer-id'
import { startDelayedInterval, moveDecimalPoint } from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import chalk from 'chalk'
import { checkPeerIdInput, isBootstrapNode, styleValue } from './utils'
import { AbstractCommand, AutoCompleteResult, GlobalState } from './abstractCommand'
import { PublicKey, Balance } from '@hoprnet/hopr-core-ethereum'

export class OpenChannel extends AbstractCommand {
  constructor(public node: Hopr) {
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
    const myAvailableTokens = await account.getBalance(true)

    if (amountToFund.lten(0)) {
      throw Error(`Invalid 'amountToFund' provided: ${amountToFund.toString(10)}`)
    } else if (amountToFund.gt(myAvailableTokens.toBN())) {
      throw Error(`You don't have enough tokens: ${amountToFund.toString(10)}<${myAvailableTokens.toBN().toString(10)}`)
    }
  }

  public async autocomplete(query: string = '', line: string = ''): Promise<AutoCompleteResult> {
    if (!query) {
      return [[this.name()], line]
    }

    const ethereum = this.node.paymentChannels
    const selfPubKey = new PublicKey(this.node.getId().pubKey.marshal())
    const self = selfPubKey.toAddress()

    const peers = this.node.getConnectedPeers().filter((p) => !isBootstrapNode(this.node, p))

    // get channels which are ours & open
    const channels = await ethereum.indexer.getChannels(async (channel) => {
      // must be one of ours
      if (!self.eq(channel.partyA) && !self.eq(channel.partyB)) return false
      // must be open
      if (channel.status === 'CLOSED') return false

      return true
    })

    // show only peers which we can see
    let availablePeers: string[] = []
    for (const peer of peers) {
      const pubKey = new PublicKey(peer.pubKey.marshal())
      const address = pubKey.toAddress()
      const hasOpenChannel = channels.some((channel) => {
        return address.eq(channel.partyA) || address.eq(channel.partyB)
      })

      if (!hasOpenChannel) availablePeers.push(peer.toB58String())
    }

    if (availablePeers.length < 1) {
      console.log(styleValue(`\nDoesn't know any new node to open a payment channel with.`, 'failure'))
      return [[''], line]
    }

    const hits = query ? availablePeers.filter((peerId: string) => peerId.startsWith(query)) : availablePeers

    return [hits.length ? hits.map((str: string) => `open ${str}`) : ['open'], line]
  }

  public async open(state: GlobalState, counterpartyStr: string, amountToFundStr: string): Promise<string> {
    let counterparty: PeerId
    try {
      counterparty = await checkPeerIdInput(counterpartyStr, state)
    } catch (err) {
      return styleValue(err.message, 'failure')
    }

    const amountToFund = new BN(moveDecimalPoint(amountToFundStr, Balance.DECIMALS))
    await this.validateAmountToFund(amountToFund)

    const unsubscribe = startDelayedInterval(`Submitted transaction. Waiting for confirmation`)
    try {
      const { channelId } = await this.node.openChannel(counterparty, amountToFund)
      unsubscribe()
      return `${chalk.green(`Successfully opened channel`)} ${styleValue(channelId.toHex(), 'hash')}`
    } catch (err) {
      unsubscribe()
      return styleValue(err.message, 'failure')
    }
  }

  /**
   * Encapsulates the functionality that is executed once the user decides to open a payment channel
   * with another party.
   * @param query peerId string to send message to
   */
  public async execute(query: string, state: GlobalState): Promise<string> {
    const [err, counterPartyB58Str, amountToFundStr] = this._assertUsage(query, [
      "counterParty's PeerId",
      'amountToFund'
    ])
    if (err) return styleValue(err, 'failure')

    return this.open(state, counterPartyB58Str, amountToFundStr)
  }
}
