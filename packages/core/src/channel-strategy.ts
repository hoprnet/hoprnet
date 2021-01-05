import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'
import BN from 'bn.js'
import { MINIMUM_REASONABLE_CHANNEL_STAKE, MAX_NEW_CHANNELS_PER_TICK, NETWORK_QUALITY_THRESHOLD } from './constants'
import debug from 'debug'
import type NetworkPeers from './network/network-peers'
const log = debug('hopr-core:channel-strategy')

export type ChannelsToOpen = [PeerId, BN]
const dest = (c: ChannelsToOpen): PeerId => c[0]
const outgoingPeer = (c: IndexerChannel): PeerId => c[0]
const indexerDest = (c: IndexerChannel): PeerId => c[1]

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
    newChannels: IndexerChannel[],
    currentChannels: IndexerChannel[],
    networkPeers: NetworkPeers,
    indexer: Indexer
  ): Promise<ChannelsToOpen[]>
  // TBD: Include ChannelsToClose as well.
}

const logChannels = (c: ChannelsToOpen[]): string => c.map((x) => x[0].toB58String() + ':' + x[1].toString()).join(', ')
const logIndexerChannels = (c: IndexerChannel[]): string =>
  c.map((x) => x[1].toB58String() + ':' + x[2].toString()).join(', ')

// Don't auto open any channels
export class PassiveStrategy implements ChannelStrategy {
  name = 'passive'

  async tick(
    _balance: BN,
    _n: IndexerChannel[],
    _c: IndexerChannel[],
    _p: NetworkPeers,
    _indexer: Indexer
  ): Promise<ChannelsToOpen[]> {
    return []
  }
}

// Open channel to as many peers as possible
export class PromiscuousStrategy implements ChannelStrategy {
  name = 'promiscuous'

  async tick(
    balance: BN,
    _n: IndexerChannel[],
    currentChannels: IndexerChannel[],
    peers: NetworkPeers,
    indexer: Indexer
  ): Promise<ChannelsToOpen[]> {
    log('currently open', logIndexerChannels(currentChannels))
    let toOpen: ChannelsToOpen[] = []

    let i = 0

    while (balance.gtn(0) && i++ < MAX_NEW_CHANNELS_PER_TICK) {
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
    return toOpen
  }
}
