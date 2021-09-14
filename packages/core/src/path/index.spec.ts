import assert from 'assert'
import { findPath } from '.'
import BN from 'bn.js'
import { Balance, PublicKey } from '@hoprnet/hopr-utils'

function checkPath(path: PublicKey[], edges: Map<PublicKey, PublicKey[]>) {
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

async function weight(c): Promise<BN> {
  return c.balance.toBN().addn(1)
}

export function fakePublicKey(i: number | string): PublicKey {
  return {
    //@ts-ignore
    id: i,
    //@ts-ignore
    eq: (x: PublicKey) => x.id == i,
    toB58String: () => i,
    toPeerId: () => {},
    toHex: () => '' + i,
    toAddress: () => 'addr' + i
  } as unknown as PublicKey
}

describe('test pathfinder with some simple topologies', function () {
  const TEST_NODES = Array.from({ length: 5 }).map((_, i) => fakePublicKey(i))
  const RELIABLE_NETWORK = (_p: any) => 1
  const UNRELIABLE_NETWORK = (p: any) => ((p.id as any) % 3 == 0 ? 0 : 1) // Node 3 is down
  const STAKE_1 = () => new Balance(new BN(1))
  // @ts-ignore
  const STAKE_N = (x: PublicKey) => new Balance(new BN(x.id + 0.1))

  // Bidirectional star, all pass through node 0
  const STAR = new Map<PublicKey, PublicKey[]>()
  STAR.set(TEST_NODES[1], [TEST_NODES[0]])
  STAR.set(TEST_NODES[2], [TEST_NODES[0]])
  STAR.set(TEST_NODES[3], [TEST_NODES[0]])
  STAR.set(TEST_NODES[4], [TEST_NODES[0]])
  STAR.set(TEST_NODES[0], [TEST_NODES[1], TEST_NODES[2], TEST_NODES[3], TEST_NODES[4]])

  const ARROW = new Map<PublicKey, PublicKey[]>()
  ARROW.set(TEST_NODES[0], [TEST_NODES[1]])
  ARROW.set(TEST_NODES[1], [TEST_NODES[2]])
  ARROW.set(TEST_NODES[2], [TEST_NODES[3]])
  ARROW.set(TEST_NODES[3], [TEST_NODES[4]])

  function fakeChannels(
    edges: Map<PublicKey, PublicKey[]>,
    stakes: (i: PublicKey) => Balance
  ): (p: PublicKey) => Promise<any[]> {
    return (a: PublicKey) =>
      Promise.resolve(
        (edges.get(a) || []).map((b) => ({
          sourcePubKey: a,
          source: a.toAddress(),
          destinationPubKey: b,
          destination: b.toAddress(),
          balance: stakes(b) as any
        }))
      )
  }

  it('should find a path through a reliable star', async function () {
    const path = await findPath(
      TEST_NODES[1],
      fakePublicKey(6),
      2,
      RELIABLE_NETWORK,
      fakeChannels(STAR, STAKE_1),
      weight
    )
    checkPath(path, STAR)
    assert(path.length == 2, 'Should find a valid acyclic path')
  })

  it('should find the most valuable path through a reliable star', async function () {
    const path = await findPath(
      TEST_NODES[1],
      fakePublicKey(6),
      2,
      RELIABLE_NETWORK,
      fakeChannels(STAR, STAKE_N),
      weight
    )
    checkPath(path, STAR)
    // @ts-ignore
    assert(path[1].id == 4, 'Last hop should be 4 (most valuable choice)')
  })

  it('should not find a path if it doesnt exist', async () => {
    let thrown = false
    try {
      await findPath(TEST_NODES[1], fakePublicKey(6), 4, RELIABLE_NETWORK, fakeChannels(STAR, STAKE_1), weight)
    } catch (e) {
      thrown = true
    }
    assert(thrown, 'should throw if there is no possible path')
  })

  it('should find a path through a reliable arrow', async () => {
    const path = await findPath(
      TEST_NODES[0],
      fakePublicKey(6),
      4,
      RELIABLE_NETWORK,
      fakeChannels(ARROW, STAKE_1),
      weight
    )
    checkPath(path, ARROW)
    assert(path.length == 4, 'Should find a valid acyclic path')
  })

  it('should not find a path if a node is unreliable', async () => {
    let thrown = false
    try {
      await findPath(TEST_NODES[0], fakePublicKey(6), 4, UNRELIABLE_NETWORK, fakeChannels(ARROW, STAKE_1), weight)
    } catch (e) {
      thrown = true
    }
    assert(thrown, 'should throw if there is no possible path')
  })
})
