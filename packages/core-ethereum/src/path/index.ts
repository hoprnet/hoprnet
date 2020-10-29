import type HoprEthereum from '../'
import {Public} from '../types'
import Heap from 'heap-js'
import {randomInteger} from '@hoprnet/hopr-utils'

type Path = Public[]

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
      (await this.coreConnector.indexer.get({partyA: start})).map((channel) => {
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
        return currentPath
      }

      const lastNode = currentPath[currentPath.length - 1]

      const newNodes = (
        await this.coreConnector.indexer.get(
          {partyA: lastNode},
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
      return []
    }
  }
}

export default PathFinder
