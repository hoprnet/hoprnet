import assert from 'assert'
import PeerId from 'peer-id'
import { findPath } from '.'
import type NetworkPeers from '../network/network-peers'
import type { Indexer } from '@hoprnet/hopr-core-connector-interface'
import BN from 'bn.js'
import { fakePeerId } from '../test-utils'

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

describe('test pathfinder with some simple topologies', function () {
  const TEST_NODES = Array.from({ length: 5 }).map((_, i) => fakePeerId(i))
  const RELIABLE_NETWORK = { qualityOf: (_p) => 1 } as NetworkPeers
  const UNRELIABLE_NETWORK = { qualityOf: (p) => ((p.id as any) % 3 == 0 ? 0 : 1) } as NetworkPeers // Node 3 is down
  const STAKE_1 = () => new BN(1)
  const STAKE_N = (x: PeerId) => new BN(x.id as unknown as number + 0.1)

  // Bidirectional star, all pass through node 0
  const STAR = new Map<PeerId, PeerId[]>()
  STAR.set(TEST_NODES[1], [TEST_NODES[0]])
  STAR.set(TEST_NODES[2], [TEST_NODES[0]])
  STAR.set(TEST_NODES[3], [TEST_NODES[0]])
  STAR.set(TEST_NODES[4], [TEST_NODES[0]])
  STAR.set(TEST_NODES[0], [TEST_NODES[1], TEST_NODES[2], TEST_NODES[3], TEST_NODES[4]])

  const ARROW = new Map<PeerId, PeerId[]>()
  ARROW.set(TEST_NODES[0], [TEST_NODES[1]])
  ARROW.set(TEST_NODES[1], [TEST_NODES[2]])
  ARROW.set(TEST_NODES[2], [TEST_NODES[3]])
  ARROW.set(TEST_NODES[3], [TEST_NODES[4]])

  function fakeIndexer(edges: Map<PeerId, PeerId[]>, stakes: (i: PeerId) => BN): Indexer {
    return {
      getChannelsFromPeer: (a: PeerId) => Promise.resolve((edges.get(a) || []).map((b) => [a, b, stakes(b) as any]))
    } as Indexer
  }

  it('should find a path through a reliable star', async function () {
    const path = await findPath(TEST_NODES[1], fakePeerId(6), 3, RELIABLE_NETWORK, fakeIndexer(STAR, STAKE_1), 0)
    checkPath(path, STAR)
    assert(path.length == 3, 'Should find a valid acyclic path')
  })

  it('should find the most valuable path through a reliable star', async function () {
    const path = await findPath(TEST_NODES[1], fakePeerId(6), 3, RELIABLE_NETWORK, fakeIndexer(STAR, STAKE_N), 0)
    checkPath(path, STAR)
    assert((path[2].id as any) == 4, 'Last hop should be 4 (most valuable choice)')
  })

  it('should not find a path if it doesnt exist', async () => {
    let thrown = false
    try {
      await findPath(TEST_NODES[1], fakePeerId(6), 4, RELIABLE_NETWORK, fakeIndexer(STAR, STAKE_1), 0)
    } catch (e) {
      thrown = true
    }
    assert(thrown, 'should throw if there is no possible path')
  })

  it('should find a path through a reliable arrow', async () => {
    const path = await findPath(TEST_NODES[0], fakePeerId(6), 5, RELIABLE_NETWORK, fakeIndexer(ARROW, STAKE_1), 0)
    checkPath(path, ARROW)
    assert(path.length == 5, 'Should find a valid acyclic path')
  })

  it('should not find a path if a node is unreliable', async () => {
    let thrown = false
    try {
      await findPath(TEST_NODES[0], fakePeerId(6), 4, UNRELIABLE_NETWORK, fakeIndexer(ARROW, STAKE_1), 0)
    } catch (e) {
      thrown = true
    }
    assert(thrown, 'should throw if there is no possible path')
  })
})
