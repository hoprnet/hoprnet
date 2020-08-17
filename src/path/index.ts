import type HoprEthereum from '../'
import { Public } from '../types'

class Path {
  constructor(private coreConnector: HoprEthereum) {}

  async findPath(start: Public, targetLength: number): Promise<Public[]> {
    const openList: Public[] = [start]
    const closedList: Public[] = []

    const cameFrom = new Map<Public, Public>()
    const fScore = new Map<Public, number>([[start, 0]])

    let current: Public = start

    while (openList.length > 0) {
      current = openList.pop() as Public

      if (fScore.get(current) == targetLength) {
        const path: Public[] = Array.from({ length: targetLength })

        for (let i = 0; i < targetLength; i++) {
          path[targetLength - i - 1] = current

          current = cameFrom.get(current)
        }

        // if (current.eq(start)) {
        //   throw Error(`Wrong path!`)
        // }

        return path
      }

      // sort according to utility function

      const add: Public[] = []

      let found: boolean
      let newNode: Public

      const _newNodes = await this.coreConnector.indexer.get({ partyA: current })

      for (let i = 0; i < _newNodes.length; i++) {
        found = false

        if (current.eq(_newNodes[i].partyA)) {
          newNode = _newNodes[i].partyB
        } else {
          newNode = _newNodes[i].partyA
        }

        for (let j = 0; j < closedList.length; j++) {
          if (closedList[j].eq(newNode)) {
            found = true
            break
          }
        }

        if (found) {
          continue
        }

        for (let j = 0; j < openList.length; j++) {
          if (openList[j].eq(newNode)) {
            found = true
            break
          }
        }

        cameFrom.set(newNode, current)
        fScore.set(newNode, fScore.get(current) + 1)

        if (!found) {
          add.push(newNode)
        }
      }

      openList.push(...add)
      closedList.push(current)
    }
  }
}

export default Path
