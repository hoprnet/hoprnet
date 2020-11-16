import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import type NetworkPeers from '../network/network-peers'

export type Path = PeerId[]

const compare = (a: Path, b: Path) => b.length - a.length

const filter = (_node: PeerId) => false

const MAX_ITERATIONS = 2000

type Channel = [PeerId, PeerId, number] // [A, B, stake]

export interface Indexer {
  getFrom(a: PeerId): Promise<Channel[]>
}

export async function findPath(
    start: PeerId,
    _destination: PeerId,
    hops: number,
    _networkPeers: NetworkPeers,
    indexer: Indexer
  ): Promise<Path>{
    /*
  const startP = new Public(start.getId().pubKey.marshal())
    const exclude = [
      destination.pubKey.marshal(),
      ...this.bootstrapServers.map((ma) => PeerId.createFromB58String(ma.getPeerId()).pubKey.marshal())
    ].map((pubKey) => new this.paymentChannels.types.Public(pubKey))
*/
/*
}

async findPath(
  start: Public,
  targetLength: number,
  maxIterations: number,
  filter?: (node: Public) => boolean
): Promise<Public[]> {
*/

  let queue = new Heap<Path>(compare)
  let iterations = 0
  

  // Preprocessing
  queue.addAll(
    (await indexer.getFrom(start)).map((channel) => {
      if (start.equals(channel[0])) {
        return [channel[1]]
      } else {
        return [channel[0]]
      }
    })
  )


  while (queue.length > 0 && iterations++ < MAX_ITERATIONS) {
    iterations++
    const currentPath = queue.peek() as Path

    if (currentPath.length == hops) {
      return currentPath
    }

    const lastNode = currentPath[currentPath.length - 1]

    const newNodes = (
      (await indexer.getFrom(lastNode)).filter((c: Channel) => !currentPath.includes(c[1]) && (filter == null || filter(c[1]))
      )
    ).map((channel) => {
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
