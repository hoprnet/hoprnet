import assert from 'assert'
import type HoprEthereum from '..'
import { randomBytes } from 'crypto'
import { Public } from '../types'

import Path from '.'

const PRIV_KEY_SIZE = 32

const NODE_COUNT = 10
const EDGE_COUNT = 10

async function generateGraph() {
  const nodes = []
  for (let i = 0; i < NODE_COUNT; i++) {
    nodes.push(await Public.fromPrivKey(randomBytes(32)))
  }

  const edges = new Map<Public, Public[]>()

  for (let i = 0; i < EDGE_COUNT; i++) {
    const a = nodes[i % NODE_COUNT]
    const b = nodes[(i + 4) % NODE_COUNT]

    const nodesFromA = edges.get(a) || []
    nodesFromA.push(b)
    edges.set(a, nodesFromA)

    const nodesFromB = edges.get(b) || []
    nodesFromB.push(a)
    edges.set(b, nodesFromB)
  }

  return { nodes, edges }
}

function generateConnector(nodes: Public[], edges: Map<Public, Public[]>) {
  const connector = ({
    indexer: {
      get({ partyA }: { partyA: Public }) {
        const connectedNodes = edges.get(partyA)

        if (connectedNodes == null) {
          return Promise.resolve([])
        }

        return connectedNodes.map((partyB) => {
          return {
            partyA,
            partyB,
          }
        })
      },
    },
  } as unknown) as HoprEthereum

  connector.path = new Path(connector)

  return connector
}

function checkGraph(path: Public[], edges: Map<Public, Public[]>) {
  for (let i = 0; i < path.length - 1; i++) {
    const edge = edges.get(path[i])
    assert(edge != null && edge.includes(path[i + 1]))
  }
}

describe('test pathfinder', function () {
  it('should find a path', async function () {
    const { nodes, edges } = await generateGraph()

    const connector = generateConnector(nodes, edges)

    const path = await connector.path.findPath(nodes[0], 4)
    checkGraph(path, edges)
  })
})
