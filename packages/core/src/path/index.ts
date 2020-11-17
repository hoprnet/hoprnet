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
  const filter = (node: PeerId) => {
    return !node.equals(destination) && networkPeers.qualityOf(node) > QUALITY_THRESHOLD
  }
  let queue = new Heap<Path>(compare)
  let iterations = 0

  // Preprocessing
  queue.addAll(
    (await indexer.getChannelsFromPeer(start)).map((channel) => {
      if (start.equals(channel[0])) {
        return [channel[1]]
      } else {
        return [channel[0]]
      }
    })
  )

  while (queue.length > 0 && iterations < MAX_ITERATIONS) {
    console.log(queue.length, iterations)
    iterations++
    const currentPath = queue.peek() as Path

    if (currentPath.length == hops) {
      return currentPath
    }

    const lastNode = currentPath[currentPath.length - 1]

    const newNodes = (await indexer.getChannelsFromPeer(lastNode))
      .filter((c: IndexerChannel) => !currentPath.includes(c[1]) && (filter == null || filter(c[1])))
      .map((channel) => {
        if (lastNode.equals(channel[0])) {
          return channel[1]
        } else {
          return channel[0]
        }
      })

    if (newNodes.length == 0) {
      queue.pop()
      continue
    }

    let nextNode: PeerId = newNodes[randomInteger(0, newNodes.length)]

    if (nextNode.equals(lastNode)) {
      if (newNodes.length == 1) {
        queue.pop()
      }
      continue
    }

    const toPush = Array.from(currentPath)
    toPush.push(nextNode)

    queue.push(toPush)
  }

  if (queue.length > 0) {
    return queue.peek()
  } else {
    throw new Error('Path not found')
  }
}
