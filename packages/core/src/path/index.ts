import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import type NetworkPeers from '../network/network-peers'
import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import Debug from 'debug'
const log = Debug('hopr-core:pathfinder')

export type Path = PeerId[]
type ChannelPath = IndexerChannel[]

const MAX_ITERATIONS = 100
const QUALITY_THRESHOLD = 0.5

const next = (c: IndexerChannel):PeerId => c[1]
const pathFrom = (c: ChannelPath): Path => [c[0][0]].concat(c.map(next))
const compare = (a: ChannelPath, b: ChannelPath) => b.length - a.length
const filterCycles = (c: IndexerChannel, p: ChannelPath): boolean => 
  !pathFrom(p).find(x => x.equals(next(c)))


export async function findPath(
  start: PeerId,
  destination: PeerId,
  hops: number,
  networkPeers: NetworkPeers,
  indexer: Indexer
): Promise<Path> {
  log('find path from', start.toB58String(), 'to ', destination.toB58String(), 'length', hops)

  const filter = (node: PeerId) => {
    return !node.equals(destination) && networkPeers.qualityOf(node) > QUALITY_THRESHOLD
  }

  let queue = new Heap<ChannelPath>(compare)
  let deadEnds = new Set<string>()
  let iterations = 0
  queue.addAll((await indexer.getChannelsFromPeer(start)).map(x => [x]))

  while (queue.length > 0 && iterations++ < MAX_ITERATIONS) {
    const currentPath = queue.peek()

    if (pathFrom(currentPath).length == hops) {
      log('Path of correct length found', 
          pathFrom(currentPath).map(x => x.toB58String()).join(','))
      return pathFrom(currentPath)
    }

    const lastPeer = next(currentPath[currentPath.length - 1])
    const newChannels = (await indexer.getChannelsFromPeer(lastPeer))
      .filter(c => filterCycles(c, currentPath)) 
      .filter(c => !deadEnds.has(next(c).toB58String()))
      .filter(c => filter(next(c)))

    if (newChannels.length == 0) {
      queue.pop()
      deadEnds.add(lastPeer.toB58String())
    } else {
      let nextChannel = newChannels[randomInteger(0, newChannels.length)]
      const toPush = Array.from(currentPath)
      toPush.push(nextChannel)
      queue.push(toPush)
    }
  }

  log('Path not found')
  throw new Error('Path not found')
}
