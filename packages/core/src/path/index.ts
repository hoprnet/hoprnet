import Heap from 'heap-js'
import { NETWORK_QUALITY_THRESHOLD, MAX_PATH_ITERATIONS } from '../constants'
import { debug } from '@hoprnet/hopr-utils'
import type { ChannelEntry, PublicKey } from '@hoprnet/hopr-utils'
import { PATH_RANDOMNESS } from '../constants'

import BN from 'bn.js'
const log = debug('hopr-core:pathfinder')

export type Path = PublicKey[]
type ChannelPath = { weight: BN; path: ChannelEntry[] }

const sum = (a: BN, b: BN) => a.add(b)
const pathFrom = (c: ChannelPath): Path => c.path.map((ce) => ce.destination) // Doesn't include ourself [0]
const filterCycles = (c: ChannelEntry, p: ChannelPath): boolean => !pathFrom(p).find((x) => x.eq(c.destination))
const rand = () => Math.random() // TODO - swap for something crypto safe
const debugPath = (p: ChannelPath) =>
  pathFrom(p)
    .map((x) => x.toString())
    .join(',')

// Weight a node based on stake, and a random component.
const defaultWeight = async (edge: ChannelEntry): Promise<BN> => {
  // Minimum is 'stake', therefore weight is monotonically increasing
  const r = 1 + rand() * PATH_RANDOMNESS
  // Log scale, but minimum 1 weight per edge
  return edge.balance.toBN().addn(1).muln(r) //log()
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
  start: PublicKey,
  destination: PublicKey,
  hops: number,
  networkQualityOf: (p: PublicKey) => number,
  getOpenChannelsFromPeer: (p: PublicKey) => Promise<ChannelEntry[]>,
  weight = defaultWeight
): Promise<Path> {
  log('find path from', start.toString(), 'to ', destination.toString(), 'length', hops)

  // Weight the path with the sum of its' edges weight
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
    if (pathFrom(currentPath).length == hops) {
      log('Path of correct length found', debugPath(currentPath), ':', currentPath.weight.toString())
      return pathFrom(currentPath)
    }

    const lastPeer = currentPath.path[currentPath.path.length - 1].destination
    const newChannels = (await getOpenChannelsFromPeer(lastPeer)).filter((c: ChannelEntry) => {
      return (
        !destination.eq(c.destination) &&
        networkQualityOf(c.destination) > NETWORK_QUALITY_THRESHOLD &&
        filterCycles(c, currentPath) &&
        !deadEnds.has(c.destination.toHex())
      )
    })

    if (newChannels.length == 0) {
      queue.pop()
      deadEnds.add(lastPeer.toHex())
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
  throw new Error('Path not found')
}
