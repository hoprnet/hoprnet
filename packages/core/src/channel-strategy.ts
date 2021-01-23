import type { Indexer, RoutingChannel } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'
import BN from 'bn.js'
import {
  MINIMUM_REASONABLE_CHANNEL_STAKE,
  MAX_NEW_CHANNELS_PER_TICK,
  NETWORK_QUALITY_THRESHOLD,
  MAX_AUTO_CHANNELS
} from './constants'
import debug from 'debug'
import type NetworkPeers from './network/network-peers'
const log = debug('hopr-core:channel-strategy')

export type ChannelsToOpen = [PeerId, BN]
export type ChannelsToClose = PeerId
const dest = (c: ChannelsToOpen): PeerId => c[0]
const outgoingPeer = (c: RoutingChannel): PeerId => c[0]
const indexerDest = (c: RoutingChannel): PeerId => c[1]

/**
 * Staked nodes will likely want to automate opening and closing of channels. By
 * implementing the following interface, they can decide how to allocate their
 * stake to best attract traffic with a useful channel graph.
 *
 * Implementors should bear in mind:
 * - Churn is expensive
 * - Path finding will prefer high stakes, and high availability of nodes.
 *
 */
export interface ChannelStrategy {
  name: string

  tick(
    balance: BN,
    newChannels: RoutingChannel[],
    currentChannels: RoutingChannel[],
    networkPeers: NetworkPeers,
    indexer: Indexer
  ): Promise<[ChannelsToOpen[], ChannelsToClose[]]>
  // TBD: Include ChannelsToClose as well.
}

const logChannels = (c: ChannelsToOpen[]): string => c.map((x) => x[0].toB58String() + ':' + x[1].toString()).join(', ')
const logIndexerChannels = (c: RoutingChannel[]): string =>
  c.map((x) => x[1].toB58String() + ':' + x[2].toString()).join(', ')

// Don't auto open any channels
export class PassiveStrategy implements ChannelStrategy {
  name = 'passive'

  async tick(
    _balance: BN,
    _n: RoutingChannel[],
    _c: RoutingChannel[],
    _p: NetworkPeers,
    _indexer: Indexer
  ): Promise<[ChannelsToOpen[], ChannelsToClose[]]> {
    return [[], []]
  }
}

// Open channel to as many peers as possible
export class PromiscuousStrategy implements ChannelStrategy {
  name = 'promiscuous'

  async tick(
    balance: BN,
    _n: RoutingChannel[],
    currentChannels: RoutingChannel[],
    peers: NetworkPeers,
    indexer: Indexer
  ): Promise<[ChannelsToOpen[], ChannelsToClose[]]> {
    log('currently open', logIndexerChannels(currentChannels))
    let toOpen: ChannelsToOpen[] = []

    let i = 0
    let toClose = currentChannels
      .filter((x: RoutingChannel) => peers.qualityOf(indexerDest(x)) < 0.1)
      .map((x) => indexerDest(x))

    while (
      balance.gtn(0) &&
      i++ < MAX_NEW_CHANNELS_PER_TICK &&
      currentChannels.length + toOpen.length < MAX_AUTO_CHANNELS
    ) {
      let randomChannel = await indexer.getRandomChannel()
      if (randomChannel === undefined) {
        log('no channel available')
        break
      }
      log('evaluating', outgoingPeer(randomChannel).toB58String())
      peers.register(outgoingPeer(randomChannel))
      if (
        !toOpen.find((x) => dest(x).equals(outgoingPeer(randomChannel))) &&
        !currentChannels.find((x) => indexerDest(x).equals(outgoingPeer(randomChannel))) &&
        peers.qualityOf(outgoingPeer(randomChannel)) > NETWORK_QUALITY_THRESHOLD
      ) {
        toOpen.push([outgoingPeer(randomChannel), MINIMUM_REASONABLE_CHANNEL_STAKE])
        balance.isub(MINIMUM_REASONABLE_CHANNEL_STAKE)
      }
    }
    log('Promiscuous toOpen: ', logChannels(toOpen))
    return [toOpen, toClose]
  }
}
