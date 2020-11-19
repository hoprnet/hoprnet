import Heap from 'heap-js'
import PeerId from 'peer-id'
import type NetworkPeers from '../network/network-peers'
import type { Indexer, IndexerChannel as Edge} from '@hoprnet/hopr-core-connector-interface'
import { NETWORK_QUALITY_THRESHOLD, MAX_PATH_ITERATIONS } from '../constants'
import Debug from 'debug'
import BN from 'bn.js'
const log = Debug('hopr-core:pathfinder')

export type Path = PeerId[]
type ChannelPath = Edge[]

const sum = (a: number, b: number) => a + b
const next = (c: Edge): PeerId => c[1]
const stake = (c: Edge): BN => c[2]
const pathFrom = (c: ChannelPath): Path => [c[0][0]].concat(c.map(next))
const filterCycles = (c: Edge, p: ChannelPath): boolean => !pathFrom(p).find((x) => x.equals(next(c)))
const rand = () => Math.random() // TODO - swap for something crypto safe
const debugPath = (p: ChannelPath) =>
  pathFrom(p)
    .map((x) => x.toB58String())
    .join(',')

/**
 * Find a path through the payment channels.
 *
 * @returns path as Array<PeerId> (including start, but not including
 * destination
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

  // Weight a node based on stake, and a random component.
  const weight = (edge: Edge): number => {
    // Minimum is 'stake', therefore weight is monotonically increasing
    const r = 1 + rand() * randomness
    // Log scale, but minimum 1 weight per edge
    return Math.log(1 + stake(edge).toNumber() * r)
  }

  const compareWeight = (a: Edge, b: Edge) => weight(b) - weight(a)

  // Weight the path with the sum of its' edges weight
  const pathWeight = (a: ChannelPath): number => a.map(weight).reduce(sum, 0)

  const comparePath = (a: ChannelPath, b: ChannelPath) => {
    return pathWeight(b) - pathWeight(a)
  }

  let queue = new Heap<ChannelPath>(comparePath)
  let deadEnds = new Set<string>()
  let iterations = 0
  queue.addAll((await indexer.getChannelsFromPeer(start)).map((x) => [x]))

  while (queue.length > 0 && iterations++ < MAX_PATH_ITERATIONS) {
    const currentPath = queue.peek()
    if (pathFrom(currentPath).length == hops) {
      log('Path of correct length found', debugPath(currentPath), ':', pathWeight(currentPath))
      return pathFrom(currentPath)
    }

    const lastPeer = next(currentPath[currentPath.length - 1])
    const newChannels = (await indexer.getChannelsFromPeer(lastPeer))
      .filter(
        (c) =>
          !destination.equals(next(c)) &&
          networkPeers.qualityOf(next(c)) > NETWORK_QUALITY_THRESHOLD &&
          filterCycles(c, currentPath) &&
          !deadEnds.has(next(c).toB58String())
      )
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
