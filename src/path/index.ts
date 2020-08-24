import type HoprEthereum from '../'
import { Public } from '../types'
import Heap from 'heap-js'
import { randomInteger } from '@hoprnet/hopr-utils'

type Path = Public[]

/**
 * returns a - b, "a without b", assuming
 * |a| >> |b|, "a much larger than b"
 * @param a
 * @param b
 */
function without(a: Public[], b: Public[], filter?: (node: Public) => boolean): Public[] {
  const toDelete: number[] = []

  for (let i = 0; i < a.length; i++) {
    if (b.includes(a[i]) || (filter != null && !filter(a[i]))) {
      toDelete.push(i)
    }
  }

  for (let i = 0; i < toDelete.length; i++) {
    a.splice(toDelete[i] - i, 1)
  }

  return a
}

class PathFinder {
  constructor(private coreConnector: HoprEthereum) {}

  async findPath(
    start: Public,
    targetLength: number,
    maxIterations: number,
    filter?: (node: Public) => boolean
  ): Promise<Public[]> {
    const compare = (a: Path, b: Path) => b.length - a.length

    let queue = new Heap<Path>(compare)

    // Preprocessing
    queue.addAll(
      (await this.coreConnector.indexer.get({ partyA: start })).map((channel) => {
        if (start.eq(channel.partyA)) {
          return [channel.partyB]
        } else {
          return [channel.partyA]
        }
      })
    )

    let iterations = 0

    while (queue.length > 0 && iterations++ < maxIterations) {
      iterations++
      const currentPath = queue.peek() as Path

      if (currentPath.length == targetLength) {
        console.log(`iterations`, iterations)
        return currentPath
      }

      const lastNode = currentPath[currentPath.length - 1]

      const newNodes = (await this.coreConnector.indexer.get({ partyA: lastNode })).map((channel) => {
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

      without(newNodes, currentPath, filter)

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
      return []
    }
  }
}

export default PathFinder
