import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import Heap from 'heap-js'
import PeerId from 'peer-id'
import BN from 'bn.js'
import { MINIMUM_REASONABLE_CHANNEL_STAKE } from './constants'

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

// Open channel to as many peers as possible
export class PromiscuousStrategy implements ChannelStrategy {
  private queue

  constructor() {
    let compare = (a, b) => b[2] - a[2] // TODO
    this.queue = new Heap<IndexerChannel>(compare)
  }

  async tick(balance: BN, indexer: Indexer): Promise<ChannelsToOpen[]> {
    let toOpen = []
    const startNode = await indexer.getRandomChannel()
    this.queue.addAll(await indexer.getChannelsFromPeer(startNode[0]))

    while (balance.gtn(0) && this.queue.length) {
      let next = this.queue.pop()[1]

      toOpen.push([next, MINIMUM_REASONABLE_CHANNEL_STAKE])
      balance.isubn(MINIMUM_REASONABLE_CHANNEL_STAKE)
      this.queue.addAll((await indexer.getChannelsFromPeer(next)).filter((x) => !toOpen.find((y) => y[0] == x)))
    }
    return toOpen
  }
}

/*
// Stake the whales
export class HarpoonStrategy implements ChannelStrategy {
  async tick(balance: BN, indexer: Indexer): Promise<ChannelsToOpen[]> {
    return []
  }
}
*/
