import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'
import BN from 'bn.js'
import { MINIMUM_REASONABLE_CHANNEL_STAKE, MAX_NEW_CHANNELS_PER_TICK } from './constants'

export type ChannelsToOpen = [PeerId, BN]

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
  async tick(balance: BN, _n, _c, indexer: Indexer): Promise<ChannelsToOpen[]> {
    let toOpen = []
    let i = 0
    while (balance.gtn(0) && i++ < MAX_NEW_CHANNELS_PER_TICK) {
      let randomChannel = await indexer.getRandomChannel()
      if (randomChannel === undefined){
        break
      }
      toOpen.push([randomChannel, MINIMUM_REASONABLE_CHANNEL_STAKE])
      balance.isubn(MINIMUM_REASONABLE_CHANNEL_STAKE)
    }
    return toOpen
  }
}
