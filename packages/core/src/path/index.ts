import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import type NetworkPeers from '../network/network-peers'
import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'
import Debug from 'debug'
const log = Debug('hopr-core:pathfinder')

export type Path = PeerId[]

const compare = (a: Path, b: Path) => b.length - a.length

const MAX_ITERATIONS = 200
const QUALITY_THRESHOLD = 0.5

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
  let queue = new Heap<Path>(compare)
  let iterations = 0
  queue.addAll((await indexer.getChannelsFromPeer(start)).map((x) => [start, x[1]]))

  while (queue.length > 0 && iterations < MAX_ITERATIONS) {
    iterations++
    const currentPath = queue.peek() as Path

    if (currentPath.length == hops) {
      log('Path of correct length found', currentPath.map(x => x.toB58String()).join(','))
      return currentPath
    }

    const lastNode = currentPath[currentPath.length - 1]

    const newNodes = (await indexer.getChannelsFromPeer(lastNode))
      .filter((c) => !currentPath.includes(c[1]))
      .filter((c) => filter(c[1]))

    if (newNodes.length == 0) {
      queue.pop()
      continue
    }

    let nextChannel: IndexerChannel = newNodes[randomInteger(0, newNodes.length)]

    const toPush = Array.from(currentPath)
    toPush.push(nextChannel[1])
    queue.push(toPush)
  }

  log('Path not found')
  throw new Error('Path not found')
}
