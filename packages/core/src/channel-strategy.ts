import type { Indexer } from '@hoprnet/hopr-core-connector-interface'
import PeerId from 'peer-id'
import BN from 'bn.js'

export type ChannelsToOpen = [PeerId, BN]

export interface ChannelStrategy {
  tick(balance: BN, indexer: Indexer): Promise<ChannelsToOpen[]>
}

// Don't auto open any channels
export class PassiveStrategy implements ChannelStrategy {
  async tick(_balance: BN, _indexer: Indexer): Promise<ChannelsToOpen[]> {
    return []
  }
}

/*
// Open channel to as many peers as possible 
export class PromiscuousStrategy implements ChannelStrategy {
  async tick(balance: BN, indexer: Indexer): Promise<ChannelsToOpen[]> {
    return []
  }
}

// Stake the whales
export class HarpoonStrategy implements ChannelStrategy {
  async tick(balance: BN, indexer: Indexer): Promise<ChannelsToOpen[]> {
    return []
  }
}
*/
