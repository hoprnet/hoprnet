import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'
import BN from 'bn.js'
import { MINIMUM_REASONABLE_CHANNEL_STAKE, MAX_NEW_CHANNELS_PER_TICK } from './constants'
import debug from 'debug'
const log = debug('hopr-core:channel-strategy')

export type ChannelsToOpen = [PeerId, BN]
const dest = (c: ChannelsToOpen) => c[0]
const outgoingPeer = (c: IndexerChannel) => c[0]
const indexerDest = (c: IndexerChannel) => c[1]

export interface ChannelStrategy {
  tick(
    balance: BN,
    newChannels: IndexerChannel[],
    currentChannels: IndexerChannel[],
    indexer: Indexer
  ): Promise<ChannelsToOpen[]>
}

// Don't auto open any channels
export class PassiveStrategy implements ChannelStrategy {
  async tick(_balance: BN, _n, _c, _indexer: Indexer): Promise<ChannelsToOpen[]> {
    return []
  }
}

// Open channel to as many peers as possible
export class PromiscuousStrategy implements ChannelStrategy {
  async tick(balance: BN, _n, currentChannels: IndexerChannel[], indexer: Indexer): Promise<ChannelsToOpen[]> {
    let toOpen = []
    let i = 0
    while (balance.gtn(0) && i++ < MAX_NEW_CHANNELS_PER_TICK) {
      let randomChannel = await indexer.getRandomChannel()
      if (randomChannel === undefined) {
        break
      }
      if (!toOpen.find(x => dest(x).equals(outgoingPeer(randomChannel))) &&
          !currentChannels.find(x => indexerDest(x).equals(outgoingPeer(randomChannel)))){
        toOpen.push([outgoingPeer(randomChannel), MINIMUM_REASONABLE_CHANNEL_STAKE])
        balance.isub(MINIMUM_REASONABLE_CHANNEL_STAKE)
      }
    }
    log('Promiscuous toOpen:\n', toOpen.map(x => x[0].toB58String() + ':' + x[1].toString()).join('\n-'))
    return toOpen
  }
}
