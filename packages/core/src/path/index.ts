import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import type NetworkPeers from '../network/network-peers'

type Path = PeerId[]

const compare = (a: Path, b: Path) => b.length - a.length
const MAX_ITERATIONS = 2000

export async function findPath(
    start: PeerId,
    destination: PeerId,
    hops: number,
    networkPeers: NetworkPeers,
    indexer: Types.Indexer
  ): Promise<Path>{
    /*
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

  const startP = new Public(start.getId().pubKey.marshal())
  let queue = new Heap<Path>(compare)
  let iterations = 0
  

  // Preprocessing
  queue.addAll(
    (await indexer.get({ partyA: start })).map((channel) => {
      if (startP.eq(channel.partyA)) {
        return [channel.partyB]
      } else {
        return [channel.partyA]
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
      await this.indexer.get(
        { partyA: lastNode },
        (node: Public) => !currentPath.includes(node) && (filter == null || filter(node))
      )
    ).map((channel) => {
      if (lastNode.eq(channel.partyA)) {
        return channel.partyB
      } else {
        return channel.partyA
      }
    })

    if (newNodes.length == 0) {
      queue.pop()
      continue
    }

    let nextNode: Public = newNodes[randomInteger(0, newNodes.length)]

    if (nextNode.eq(lastNode)) {
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
