import assert from 'assert'
import { findPath } from './index.js'
import BN from 'bn.js'
import { Balance, BalanceType, type ChannelEntry, PublicKey, stringToU8a } from '@hoprnet/hopr-utils'

const pubKeys = [
  '0x0443a3958ac66a3b2ab89fcf90bc948a8b8be0e0478d21574d077ddeb11f4b1e9f2ca21d90bd66cee037255480a514b91afae89e20f7f7fa7353891cc90a52bf6e',
  '0x04f16fd6701aea01032716377d52d8213497c118f99cdd1c3c621b2795cac8681606b7221f32a8c5d2ef77aa783bec8d96c11480acccabba9e8ee324ae2dfe92bb',
  '0x04613ec6ef4bf62b9b1132d3f515e51cc6cb4101c4ad6b63b8a18915257985dbdf035156e2a35a9818ccfa263c5777e77ce5cbb7f71ae4275c4accb4d72c010985',
  '0x045b1e46f70628d6ab2436b4dd6120a85dff3ef3077035a1eb07f09f96f47e9ed7876d22fb3cdf6232f89e31080c96f075d0b2819be990206076bbae9486b1b3b6',
  '0x047610d3509dcfd799132470a812cd2862259f711ebcde5b3057a4a18beb9fa79a0ca3c367e6b79fcb13f72adca90c6af0788fc57c903f80175dac11037f4a485c',
  '0x04c4d09dbf7233bdc7e27d7ef7f13c924a8dc95f295ef462484cff03030478b18de0e4cae3d9e1d4280ef3aded0f8d366f10f4513482d3972221a8586bf0dce439'
]

const TEST_NODES = Array.from({ length: 6 }, (_, index: number) => PublicKey.deserialize(stringToU8a(pubKeys[index])))

function testNodeId(pKey: PublicKey) {
  return TEST_NODES.findIndex((testKey) => testKey.eq(pKey))
}

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

async function weight(c: ChannelEntry): Promise<BN> {
  return new BN(c.balance.to_string()).addn(1)
}

describe('test pathfinder with some simple topologies', function () {
  const RELIABLE_NETWORK = (_p: PublicKey) => 1
  const UNRELIABLE_NETWORK = (pubKey: PublicKey) => (testNodeId(pubKey) % 3 == 0 ? 0 : 1) // Node 3 is down
  const STAKE_1 = () => new Balance('1', BalanceType.HOPR)

  const STAKE_N = (pubKey: PublicKey) => new Balance((testNodeId(pubKey) + 1).toString(10), BalanceType.HOPR)

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

  function fakeChannels(
    edges: Map<PublicKey, PublicKey[]>,
    stakes: (i: PublicKey) => Balance
  ): (p: PublicKey) => Promise<ChannelEntry[]> {
    return (a: PublicKey) =>
      Promise.resolve(
        (edges.get(a) || []).map((b) => ({ source: a, destination: b, balance: stakes(b) })) as ChannelEntry[]
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
