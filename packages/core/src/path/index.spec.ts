import assert from 'assert'
//import { randomSubset } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { findPath } from '.'
import type NetworkPeers from '../network/network-peers'
import type { Indexer } from '@hoprnet/hopr-core-connector-interface'

function fakePeerId(i: number): PeerId {
  return ({
    id: i,
    equals: (x) => x.id == i
  } as unknown) as PeerId
}
/*
async function generateRandomGraph(nodesCount: number) {
  const nodes = []

  for (let i = 0; i < nodesCount; i++) {
    nodes.push(fakePeerId(i))
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
}*/

function checkPath(path: PeerId[], edges: Map<PeerId, PeerId[]>) {
  for (let i = 0; i < path.length - 1; i++) {
    const edgeSet = edges.get(path[i])
    if (edgeSet == null) {
      throw new Error('Invalid path missing edge ' + i)
    }
    if (!edgeSet.includes(path[i + 1])) {
      throw new Error('Invalid path, next edge missing ' + i)
    }
    if ((i > 0 && path.slice(0, i).includes(path[i])) || path.slice(i + 1).includes(path[i])) {
      throw new Error('Invalid path - contains cycle')
    }
  }
}

const TEST_NODES = Array.from({ length: 5 }).map((_, i) => fakePeerId(i))

// Bidirectional star, all pass through node 0
const STAR = new Map<PeerId, PeerId[]>()
STAR.set(TEST_NODES[1], [TEST_NODES[0]])
STAR.set(TEST_NODES[2], [TEST_NODES[0]])
STAR.set(TEST_NODES[3], [TEST_NODES[0]])
STAR.set(TEST_NODES[4], [TEST_NODES[0]])
STAR.set(TEST_NODES[0], [TEST_NODES[1], TEST_NODES[2], TEST_NODES[3], TEST_NODES[4]])

const RELIABLE_NETWORK = { qualityOf: (_p) => 1 } as NetworkPeers
const STAKE_1 = () => 1

function fakeIndexer(edges: Map<PeerId, PeerId[]>, stakes: (i: PeerId) => number): Indexer {
  return {
    getChannelsFromPeer: (a: PeerId) => Promise.resolve(edges.get(a).map((b) => [a, b, stakes(a) as any]))
  }
}

describe('test pathfinder', function () {
  it('should find a path through a reliable star', async function () {
    const path = await findPath(TEST_NODES[1], fakePeerId(6), 3, RELIABLE_NETWORK, fakeIndexer(STAR, STAKE_1))
    checkPath(path, STAR)
    assert(path.length == 3, 'Should find a valid acyclic path')
  })
})
