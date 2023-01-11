import HoprCoreEthereum, { type ChannelEntry } from '@hoprnet/hopr-core-ethereum'
import {
  type AcknowledgedTicket,
  PublicKey,
  MINIMUM_REASONABLE_CHANNEL_STAKE,
  MAX_AUTO_CHANNELS,
  PRICE_PER_PACKET,
  debug,
  Balance
} from '@hoprnet/hopr-utils'
import BN from 'bn.js'
import { MAX_NEW_CHANNELS_PER_TICK, NETWORK_QUALITY_THRESHOLD, INTERMEDIATE_HOPS, CHECK_TIMEOUT } from './constants.js'
import type NetworkPeers from './network/network-peers.js'
import { NetworkPeersOrigin } from './network/network-peers.js'
import type { PeerId } from '@libp2p/interface-peer-id'

const log = debug('hopr-core:channel-strategy')

export type StrategyTickResult = {
  toOpen: {
    destination: string
    stake: BigInt
  }[]
  toClose: string[]
}

/**
 * Staked nodes will likely want to automate opening and closing of channels. By
 * implementing the following interface, they can decide how to allocate their
 * stake to best attract traffic with a useful channel graph.
 *
 * Implementors should bear in mind:
 * - Churn is expensive
 * - Path finding will prefer high stakes, and high availability of nodes.
 */
export interface ChannelStrategyInterface {
  name: string

  tick(
    balance: Balance,
    networkSize: number,
    currentChannels: ChannelEntry[],
    qualityOf: (peerId: PeerId) => number,
    peers: Iterable<PeerId>
  ): Promise<StrategyTickResult>
  // TBD: Include ChannelsToClose as well.

  onChannelWillClose(channel: ChannelEntry, chain: HoprCoreEthereum): Promise<void> // Before a channel closes
  onWinningTicket(t: AcknowledgedTicket, chain: HoprCoreEthereum): Promise<void>
  shouldCommitToChannel(c: ChannelEntry): Promise<boolean>

  tickInterval: number
}

/*
 * Saves duplication of 'normal' behaviour.
 *
 * At present this does not take gas into consideration.
 */
export abstract class SaneDefaults {
  async onWinningTicket(ackTicket: AcknowledgedTicket, chain: HoprCoreEthereum) {
    const counterparty = ackTicket.signer
    log(`auto redeeming tickets in channel to ${counterparty.toPeerId().toString()}`)
    await chain.redeemTicketsInChannelByCounterparty(counterparty)
  }

  async onChannelWillClose(channel: ChannelEntry, chain: HoprCoreEthereum) {
    const counterparty = channel.source
    const selfPubKey = chain.getPublicKey()
    if (!counterparty.eq(selfPubKey)) {
      log(`auto redeeming tickets in channel to ${counterparty.toPeerId().toString()}`)
      try {
        await chain.redeemTicketsInChannel(channel)
      } catch (err) {
        log(`Could not redeem tickets in channel ${channel.getId().toHex()}`, err)
      }
    }
  }

  async shouldCommitToChannel(c: ChannelEntry): Promise<boolean> {
    log(`committing to channel ${c.getId().toHex()}`)
    return true
  }

  tickInterval = CHECK_TIMEOUT
}

// Don't auto open any channels
export class PassiveStrategy extends SaneDefaults implements ChannelStrategyInterface {
  name = 'passive'

  async tick(_balance: BN, _c: ChannelEntry[], _p: NetworkPeers): Promise<StrategyTickResult> {
    return { toOpen: [], toClose: [] }
  }
}

// Open channel to as many peers as possible
export class PromiscuousStrategy extends SaneDefaults implements ChannelStrategyInterface {
  name = 'promiscuous'

  async tick(
    balance: BN,
    currentChannels: ChannelEntry[],
    peers: NetworkPeers,
    getRandomChannel: () => Promise<ChannelEntry>
  ): Promise<StrategyTickResult> {
    log(
      'currently open',
      currentChannels.map((x) => x.toString())
    )
    let toOpen: StrategyTickResult['toOpen'] = []

    let i = 0
    let toClose = currentChannels.filter((x: ChannelEntry) => {
      return (
        peers.qualityOf(x.destination.toPeerId()) < 0.1 ||
        // Lets append channels with less balance than a full hop messageto toClose.
        // NB: This is based on channel balance, not expected balance so may not be
        // aggressive enough.
        x.balance.toBN().lte(PRICE_PER_PACKET.muln(INTERMEDIATE_HOPS))
      )
    })

    // First let's open channels to any interesting peers we have
    peers.all().forEach((peerId) => {
      if (
        balance.gtn(0) &&
        currentChannels.length + toOpen.length < MAX_AUTO_CHANNELS &&
        !toOpen.find((x: typeof toOpen[number]) => x.destination.eq(PublicKey.fromPeerId(peerId))) &&
        !currentChannels.find((x) => x.destination.toPeerId().equals(peerId)) &&
        peers.qualityOf(peerId) > NETWORK_QUALITY_THRESHOLD
      ) {
        toOpen.push({
          destination: PublicKey.fromPeerId(peerId),
          stake: MINIMUM_REASONABLE_CHANNEL_STAKE
        })
        balance.isub(MINIMUM_REASONABLE_CHANNEL_STAKE)
      }
    })

    // Now let's evaluate new channels
    while (
      balance.gtn(0) &&
      i++ < MAX_NEW_CHANNELS_PER_TICK &&
      currentChannels.length + toOpen.length < MAX_AUTO_CHANNELS
    ) {
      let randomChannel = await getRandomChannel()
      if (randomChannel === undefined) {
        log('no channel available')
        break
      }
      log('evaluating', randomChannel.source.toString())
      peers.register(randomChannel.source.toPeerId(), NetworkPeersOrigin.STRATEGY_CONSIDERING_CHANNEL)
      if (
        !toOpen.find((x) => x[0].eq(randomChannel.source)) &&
        !currentChannels.find((x) => x.destination.eq(randomChannel.source)) &&
        peers.qualityOf(randomChannel.source.toPeerId()) > NETWORK_QUALITY_THRESHOLD
      ) {
        toOpen.push({
          destination: randomChannel.source,
          stake: MINIMUM_REASONABLE_CHANNEL_STAKE
        })
        balance.isub(MINIMUM_REASONABLE_CHANNEL_STAKE)
      }
    }
    log(
      'Promiscuous toOpen: ',
      toOpen.map((p) => p.toString())
    )
    return { toOpen, toClose }
  }
}
