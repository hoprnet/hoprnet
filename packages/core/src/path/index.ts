import HeapPackage from 'heap-js'
import { NETWORK_QUALITY_THRESHOLD, MAX_PATH_ITERATIONS, PATH_RANDOMNESS, MAX_HOPS } from '../constants.js'
import { type ChannelEntry, type Address } from '@hoprnet/hopr-utils'
import { debug, random_float } from '@hoprnet/hopr-utils'

import BN from 'bn.js'

const { Heap } = HeapPackage

const log = debug('hopr-core:pathfinder')

export type Path = Address[]
type ChannelPath = { weight: BN; path: ChannelEntry[] }

const sum = (a: BN, b: BN) => a.add(b)
const pathFrom = (c: ChannelPath): Path => c.path.map((ce) => ce.destination) // Doesn't include ourself [0]
const filterCycles = (c: ChannelEntry, p: ChannelPath): boolean => !pathFrom(p).find((x) => x.eq(c.destination))
const debugPath = (p: ChannelPath) =>
  pathFrom(p)
    .map((x) => x.toString())
    .join(',')

// Weight a node based on stake, and a random component.
const defaultWeight = async (edge: ChannelEntry): Promise<BN> => {
  // Minimum is 'stake', therefore weight is monotonically increasing
  const r = 1 + random_float() * PATH_RANDOMNESS
  // Log scale, but minimum 1 weight per edge
  return new BN(edge.balance.to_string(), 10).addn(1).muln(r) //log()
}

/**
 * Find a path through the payment channels.
 *
 * Depth first search through potential paths based on weight
 *
 * @returns path as Array<PeerId> (including start, but not including
 * destination
 */
export async function findPath(
  start: Address,
  destination: Address,
  hops: number,
  networkQualityOf: (p: Address) => Promise<number>,
  getOpenChannelsFromPeer: (p: Address) => Promise<ChannelEntry[]>,
  weight = defaultWeight
): Promise<Path> {
  log('find path from', start.toString(), 'to ', destination.toString(), 'length', hops)

  // Weight the path with the sum of its edges weight
  const pathWeight = async (a: ChannelEntry[]): Promise<BN> => (await Promise.all(a.map(weight))).reduce(sum, new BN(0))

  const comparePath = (a: ChannelPath, b: ChannelPath): number => {
    return b.weight.gte(a.weight) ? 1 : -1
  }

  let queue = new Heap<ChannelPath>(comparePath)
  let deadEnds = new Set<string>()
  let iterations = 0
  let initialChannels = await getOpenChannelsFromPeer(start)
  await Promise.all(initialChannels.map(async (x) => queue.add({ weight: await weight(x), path: [x] })))

  while (queue.length > 0 && iterations++ < MAX_PATH_ITERATIONS) {
    const currentPath: ChannelPath = queue.peek()
    const currPathLen = pathFrom(currentPath).length
    if (currPathLen >= hops && currPathLen <= MAX_HOPS) {
      log('Path of correct length found', debugPath(currentPath), ':', currentPath.weight.toString())
      return pathFrom(currentPath)
    }

    const lastPeer = currentPath.path[currentPath.path.length - 1].destination
    const openChannels = await getOpenChannelsFromPeer(lastPeer)

    const newChannels = []

    for (const openChannel of openChannels) {
      if (
        !destination.eq(openChannel.destination) &&
        (await networkQualityOf(openChannel.destination)) > NETWORK_QUALITY_THRESHOLD &&
        filterCycles(openChannel, currentPath) &&
        !deadEnds.has(openChannel.destination.to_hex())
      ) {
        newChannels.push(openChannel)
      }
    }

    if (newChannels.length == 0) {
      queue.pop()
      deadEnds.add(lastPeer.to_hex())
    } else {
      for (let c of newChannels) {
        const toPush = Array.from(currentPath.path)
        toPush.push(c)
        const w = await pathWeight(toPush)
        queue.push({ weight: w, path: toPush })
      }
    }
  }

  log('Path not found')
  throw Error('Failed to find automatic path')
}
