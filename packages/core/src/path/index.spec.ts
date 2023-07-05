import assert from 'assert'
import { findPath } from './index.js'
import BN from 'bn.js'
import { Balance, BalanceType, type ChannelEntry, Address, stringToU8a } from '@hoprnet/hopr-utils'

const addrs = [
  '0xafe8c178cf70d966be0a798e666ce2782c7b2288',
  '0x1223d5786d9e6799b3297da1ad55605b91e2c882',
  '0x0e3e60ddced1e33c9647a71f4fc2cf4ed33e4a9d',
  '0x27644105095c8c10f804109b4d1199a9ac40ed46',
  '0x4701a288c38fa8a0f4b79127747257af4a03a623',
  '0xfddd2f462ec709cf181bbe44a7e952487bd4591d'
]

const TEST_NODES = Array.from({ length: addrs.length }, (_, index: number) =>
  Address.deserialize(stringToU8a(addrs[index]))
)

function testNodeId(pKey: Address) {
  return TEST_NODES.findIndex((testKey) => testKey.eq(pKey))
}

function checkPath(path: Address[], edges: Map<Address, Address[]>) {
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

async function weight(c: ChannelEntry): Promise<BN> {
  return new BN(c.balance.to_string()).addn(1)
}

describe('test pathfinder with some simple topologies', function () {
  const RELIABLE_NETWORK = async (_p: Address) => 1
  const UNRELIABLE_NETWORK = async (address: Address) => (testNodeId(address) % 3 == 0 ? 0 : 1) // Node 3 is down
  const STAKE_1 = () => new Balance('1', BalanceType.HOPR)

  const STAKE_N = (pubKey: Address) => new Balance((testNodeId(pubKey) + 1).toString(10), BalanceType.HOPR)

  // Bidirectional star, all pass through node 0
  const STAR = new Map<Address, Address[]>()
  STAR.set(TEST_NODES[1], [TEST_NODES[0]])
  STAR.set(TEST_NODES[2], [TEST_NODES[0]])
  STAR.set(TEST_NODES[3], [TEST_NODES[0]])
  STAR.set(TEST_NODES[4], [TEST_NODES[0]])
  STAR.set(TEST_NODES[0], [TEST_NODES[1], TEST_NODES[2], TEST_NODES[3], TEST_NODES[4]])

  const ARROW = new Map<Address, Address[]>()
  ARROW.set(TEST_NODES[0], [TEST_NODES[1]])
  ARROW.set(TEST_NODES[1], [TEST_NODES[2]])
  ARROW.set(TEST_NODES[2], [TEST_NODES[3]])

  function fakeChannels(
    edges: Map<Address, Address[]>,
    stakes: (i: Address) => Balance
  ): (p: Address) => Promise<ChannelEntry[]> {
    return (a: Address) =>
      Promise.resolve(
        (edges.get(a) || []).map((b) => ({
          source: a,
          destination: b,
          balance: stakes(b)
        })) as ChannelEntry[]
      )
  }

  it('should find a path through a reliable star', async function () {
    const path = await findPath(TEST_NODES[1], TEST_NODES[5], 2, RELIABLE_NETWORK, fakeChannels(STAR, STAKE_1), weight)
    checkPath(path, STAR)
    assert(path.length == 2, 'Should find a valid acyclic path')
  })

  it('should find the most valuable path through a reliable star', async function () {
    const path = await findPath(TEST_NODES[1], TEST_NODES[5], 2, RELIABLE_NETWORK, fakeChannels(STAR, STAKE_N), weight)
    checkPath(path, STAR)
    // @ts-ignore
    assert(testNodeId(path[1]) == 4, 'Last hop should be 4 (most valuable choice)')
  })

  it('should not find a path if it doesnt exist', async () => {
    await assert.rejects(
      async () =>
        await findPath(TEST_NODES[1], TEST_NODES[5], 4, RELIABLE_NETWORK, fakeChannels(STAR, STAKE_1), weight),
      'should throw if there is no possible path'
    )
  })

  it('should find a path through a reliable arrow', async () => {
    const path = await findPath(TEST_NODES[0], TEST_NODES[5], 3, RELIABLE_NETWORK, fakeChannels(ARROW, STAKE_1), weight)
    checkPath(path, ARROW)
    assert(path.length == 3, 'Should find a valid acyclic path')
  })

  it('should not find a path if a node is unreliable', async () => {
    await assert.rejects(
      async () =>
        await findPath(TEST_NODES[0], TEST_NODES[5], 3, UNRELIABLE_NETWORK, fakeChannels(ARROW, STAKE_1), weight),
      'should throw if there is no possible path'
    )
  })
})
