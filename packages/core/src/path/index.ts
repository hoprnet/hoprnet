import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import type NetworkPeers from '../network/network-peers'
import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import Debug from 'debug'
const log = Debug('hopr-core:pathfinder')

export type Path = PeerId[]
type ChannelPath = IndexerChannel[]
type Edge = IndexerChannel

// TODO move to consts
const MAX_ITERATIONS = 100
const QUALITY_THRESHOLD = 0.5

const sum = (a: number, b: number) => a + b
const next = (c: Edge): PeerId => c[1]
const stake = (c: Edge): number => c[2]
const pathFrom = (c: ChannelPath): Path => [c[0][0]].concat(c.map(next))
const filterCycles = (c: Edge, p: ChannelPath): boolean => !pathFrom(p).find((x) => x.equals(next(c)))

/*
 * Find a path through the payment channels.
 *
 */
export async function findPath(
  start: PeerId,
  destination: PeerId,
  hops: number,
  networkPeers: NetworkPeers,
  indexer: Indexer,
  randomness: number // Proportion of randomness in stake.
): Promise<Path> {
  log('find path from', start.toB58String(), 'to ', destination.toB58String(), 'length', hops)

  // Discard destination or low QoS nodes.
  const filter = (node: PeerId) => !node.equals(destination) && networkPeers.qualityOf(node) > QUALITY_THRESHOLD

  // Weight a node based on stake, and a random component.
  const weight = (edge: Edge): number => {
    const rand = randomness > 0 ? randomInteger(0, randomness) : 1 // TODO use float
    return stake(edge) * rand 
  }

  const compareWeight = (a: Edge, b: Edge) => weight(b) - weight(a)

  // Weight the path with the sum of it's edges weight
  const pathWeight = (a: ChannelPath): number => a.map(weight).reduce(sum, 0)

  const comparePath = (a: ChannelPath, b: ChannelPath) => {
    console.log(pathWeight(a), pathWeight(b))
    return pathWeight(b) - pathWeight(a) 
  }

  let queue = new Heap<ChannelPath>(comparePath)
  let deadEnds = new Set<string>()
  let iterations = 0
  queue.addAll((await indexer.getChannelsFromPeer(start)).map((x) => [x]))

  while (queue.length > 0 && iterations++ < MAX_ITERATIONS) {
    const currentPath = queue.peek()
    log('current path', pathFrom(currentPath)
          .map((x) => x.toB58String())
          .join(','), 'weight', pathWeight(currentPath))

    if (pathFrom(currentPath).length == hops) {
      log(
        'Path of correct length found',
        pathFrom(currentPath)
          .map((x) => x.toB58String())
          .join(',')
      )
      return pathFrom(currentPath)
    }

    const lastPeer = next(currentPath[currentPath.length - 1])
    const newChannels = (await indexer.getChannelsFromPeer(lastPeer))
      .filter((c) => filterCycles(c, currentPath))
      .filter((c) => !deadEnds.has(next(c).toB58String()))
      .filter((c) => filter(next(c)))
      .sort(compareWeight)

    if (newChannels.length == 0) {
      queue.pop()
      deadEnds.add(lastPeer.toB58String())
    } else {
      const toPush = Array.from(currentPath)
      toPush.push(newChannels[0])
      queue.push(toPush)
    }
  }

  log('Path not found')
  throw new Error('Path not found')
}
