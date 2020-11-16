import assert from 'assert'
import type HoprEthereum from '..'
import { randomBytes } from 'crypto'
import { gcd } from '@hoprnet/hopr-utils'
import type { Path } from '.'

function findGenerator(nodesCount: number, previousGenerator?: number) {
  for (let i = previousGenerator != null ? previousGenerator + 1 : 2; i < nodesCount; i++) {
    if (gcd(i, nodesCount) == 1) {
      return i
    }
  }
  return -1
}

async function generateGraph(nodesCount: number) {
  const nodes = []

  for (let i = 0; i < nodesCount; i++) {
    nodes.push(await Public.fromPrivKey(randomBytes(32)))
  }

  const edges = new Map<Public, Public[]>()

  if (nodesCount <= 1) {
    return { nodes, edges }
  }

  if (nodesCount == 2) {
    edges.set(nodes[0], [nodes[1]])
    edges.set(nodes[1], [nodes[0]])

    return { nodes, edges }
  }

  // find generators
  let generator = findGenerator(nodesCount)
  let secondGenerator = findGenerator(nodesCount, generator)
  let thirdGenerator = findGenerator(nodesCount, secondGenerator)

  if (generator < 0) {
    throw Error(`Failed to find generator`)
  }

  // This should generate a fully connected network
  for (let i = 0; i < nodesCount; i++) {
    const a = nodes[i % nodesCount]
    const b = nodes[(i + generator) % nodesCount]
    const c = nodes[(i + secondGenerator) % nodesCount]
    const d = nodes[(i + thirdGenerator) % nodesCount]

    const nodesFromA = edges.get(a) || []
    nodesFromA.push(b, c, d)
    edges.set(a, nodesFromA)
  }

  return { nodes, edges }
}

function generateConnector(edges: Map<Public, Public[]>) {
  const connector = ({
    indexer: {
      get({ partyA }: { partyA: Public }, filter?: (node: Public) => boolean) {
        let connectedNodes = edges.get(partyA)

        if (filter != null) {
          connectedNodes = connectedNodes.filter(filter)
        }

        if (connectedNodes == null) {
          return Promise.resolve([])
        }

        return connectedNodes.map((partyB) => {
          return {
            partyA,
            partyB
          }
        })
      }
    }
  } as unknown) as HoprEthereum

  connector.path = new Path(connector)

  return connector
}

function validPath(path: Public[], edges: Map<Public, Public[]>) {
  for (let i = 0; i < path.length - 1; i++) {
    const edgeSet = edges.get(path[i])

    if (edgeSet == null || !edgeSet.includes(path[i + 1])) {
      return false
    }
  }

  return true
}

function noCircles(path: Public[]) {
  for (let i = 1; i < path.length; i++) {
    if (path.slice(0, i).includes(path[i]) || path.slice(i + 1).includes(path[i])) {
      return false
    }
  }

  return true
}

describe('test pathfinder', function () {
  it('should find a path', async function () {
    const { nodes, edges } = await generateGraph(101)

    const connector = generateConnector(edges)

    const path = await findPath(nodes[0], undefined, 8, undefined, connector)

    assert(
      path.length == 8 && noCircles(path) && validPath(path, edges),
      'Should find a valid acyclic path that goes through all nodes'
    )
  })
})
