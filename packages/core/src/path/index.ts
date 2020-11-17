import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import type NetworkPeers from '../network/network-peers'
import type { Indexer, IndexerChannel } from '@hoprnet/hopr-core-connector-interface'

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
  console.log('find path from', start, 'to ', destination, 'length', hops)
  const filter = (node: PeerId) => {
    return !node.equals(destination) && networkPeers.qualityOf(node) > QUALITY_THRESHOLD
  }
  let queue = new Heap<Path>(compare)
  let iterations = 0
  queue.addAll((await indexer.getChannelsFromPeer(start)).map((x) => [start, x[1]]))

  while (queue.length > 0 && iterations < MAX_ITERATIONS) {
    console.log(queue.length, iterations)
    iterations++
    const currentPath = queue.peek() as Path
    console.log('>>', currentPath)

    if (currentPath.length == hops) {
      return currentPath
    }

    const lastNode = currentPath[currentPath.length - 1]
    console.log('->', lastNode)

    const newNodes = (await indexer.getChannelsFromPeer(lastNode))
      .filter((c) => !currentPath.includes(c[1]))
      .filter((c) => filter(c[1]))

    console.log('>>', newNodes.length)

    if (newNodes.length == 0) {
      queue.pop()
      continue
    }

    let nextChannel: IndexerChannel = newNodes[randomInteger(0, newNodes.length)]

    const toPush = Array.from(currentPath)
    toPush.push(nextChannel[1])
    queue.push(toPush)
  }
  throw new Error('Path not found')
}
