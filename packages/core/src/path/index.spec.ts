import assert from 'assert'
import { randomSubset } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { findPath } from '.'
import type NetworkPeers from '../network/network-peers'

async function generateGraph(nodesCount: number) {
  const nodes = []

  for (let i = 0; i < nodesCount; i++) {
    nodes.push(await PeerId.create({ bits: 512 }))
  }

  const edges = new Map<PeerId, PeerId[]>()

  console.log('add edges')

  // Random graph
  nodes.forEach((n) => {
    edges.set(
      n,
      randomSubset(nodes, 5).filter((x) => !x.equals(n))
    )
  })
  return { nodes, edges }
}

function validPath(path: PeerId[], edges: Map<PeerId, PeerId[]>) {
  for (let i = 0; i < path.length - 1; i++) {
    const edgeSet = edges.get(path[i])

    if (edgeSet == null || !edgeSet.includes(path[i + 1])) {
      return false
    }
  }

  return true
}

function noCircles(path: PeerId[]) {
  for (let i = 1; i < path.length; i++) {
    if (path.slice(0, i).includes(path[i]) || path.slice(i + 1).includes(path[i])) {
      return false
    }
  }

  return true
}

describe('test pathfinder', function () {
  it('should find a path', async function () {
    const { nodes, edges } = await generateGraph(10)
    const dest = await PeerId.create()
    const indexer = {
      getChannelsFromPeer: (a: PeerId) => Promise.resolve(edges.get(a).map((b) => [a, b, 0]))
    } as any
    const mockNetwork = { qualityOf: (_p) => 1 } as NetworkPeers
    const path = await findPath(nodes[0], dest, 3, mockNetwork, indexer)

    assert(
      path.length == 3 && noCircles(path) && validPath(path, edges),
      'Should find a valid acyclic path that goes through all nodes'
    )
  })
})
