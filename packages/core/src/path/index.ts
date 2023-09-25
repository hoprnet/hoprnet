import HeapPackage from 'heap-js'
import BN from 'bn.js'

import { debug, random_float, ChannelEntry } from '@hoprnet/hopr-utils'

import { NETWORK_QUALITY_THRESHOLD, MAX_PATH_ITERATIONS, PATH_RANDOMNESS, MAX_HOPS } from '../constants.js'

import type { Address } from '@hoprnet/hopr-utils'

const { Heap } = HeapPackage

const log = debug('hopr-core:pathfinder')

export type Path = Address[]
type ChannelPath = { weight: BN; path: ChannelEntry[] }

const sum = (a: BN, b: BN) => a.add(b)
const pathFrom = (c: ChannelPath): Path => c.path.map((ce) => ce.destination) // Doesn't include ourself [0]
const filterCycles = (c: ChannelEntry, p: ChannelPath): boolean => {
  if (p) {
    return !pathFrom(p).find((x) => x.eq(c.destination))
  }
  return true
}
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

// Filter given channels by a set of criteria to get good paths.
async function filterChannels(
  channels: ChannelEntry[],
  destination: Address,
  currentPath: ChannelPath,
  deadEnds: Set<string>,
  networkQualityOf: (p: Address) => Promise<number>
): Promise<ChannelEntry[]> {
  return (
    await Promise.all(
      channels.map(async (c): Promise<[boolean, ChannelEntry]> => {
        const valid =
          !destination.eq(c.destination) &&
          (await networkQualityOf(c.destination)) > NETWORK_QUALITY_THRESHOLD &&
          filterCycles(c, currentPath) &&
          !deadEnds.has(c.destination.to_hex())
        return [valid, c]
      })
    )
  )
    .filter(([v, _c]) => v)
    .map(([_v, c]) => c)
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
  let currentPath: ChannelPath = undefined
  let iterations = 0
  let initialChannels = await filterChannels(
    await getOpenChannelsFromPeer(start),
    destination,
    currentPath,
    deadEnds,
    networkQualityOf
  )
  await Promise.all(initialChannels.map(async (x) => queue.add({ weight: await weight(x), path: [x] })))

  while (queue.length > 0 && iterations++ < MAX_PATH_ITERATIONS) {
    currentPath = queue.peek()
    const currPathLen = pathFrom(currentPath).length
    if (currPathLen >= hops && currPathLen <= MAX_HOPS) {
      log('Path of correct length found', debugPath(currentPath), ':', currentPath.weight.toString())
      return pathFrom(currentPath)
    }

    const lastPeer = currentPath.path[currentPath.path.length - 1].destination
    const openChannels = await getOpenChannelsFromPeer(lastPeer)

    const newChannels: ChannelEntry[] = []
    const usefuleOpenChannels: ChannelEntry[] = await filterChannels(
      openChannels,
      destination,
      currentPath,
      deadEnds,
      networkQualityOf
    )
    usefuleOpenChannels.forEach((c) => newChannels.push(c))

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
