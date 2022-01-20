import { NOISE } from '@chainsafe/libp2p-noise'
import MPLEX from 'libp2p-mplex'
import Libp2p from 'libp2p'
import { dial as dialHelper, DialStatus } from './dialHelper'
import { privKeyToPeerId } from './privKeyToPeerId'
import TCP from 'libp2p-tcp'
import KadDHT from 'libp2p-kad-dht'
import assert from 'assert'
import { Multiaddr } from 'multiaddr'
import pipe from 'it-pipe'
import { u8aEquals, stringToU8a } from '../u8a'
import { createRelayerKey } from '../libp2p'
import PeerId from 'peer-id'

const TEST_PROTOCOL = '/test'
const TEST_MESSAGE = new TextEncoder().encode('test msg')

const Alice = privKeyToPeerId(stringToU8a('0xcf0b158c5f9d83dabf81a43391cce6cced6d0f912ed7152fc8b67dcdae9db591'))
const Bob = privKeyToPeerId(stringToU8a('0x801f499e287fa0e5ac546a86d7f1e3ca766249f62759e6a1f2c90de6090cc4c0'))
const Chris = privKeyToPeerId(stringToU8a('0x1bbb9a915ddd6e19d0f533da6c0fbe8820541a370110728f647829cd2c91bc79'))

async function getNode(id: PeerId, withDHT = false, maDestination?: Multiaddr): Promise<Libp2p> {
  const node = await Libp2p.create({
    addresses: {
      listen: [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${id.toB58String()}`).toString()]
    },
    peerId: id,
    modules: {
      transport: [TCP],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE],
      dht: withDHT ? KadDHT : undefined
    },
    metrics: {
      enabled: false
    },
    config: {
      dht: {
        enabled: withDHT
      },
      nat: {
        enabled: false
      },
      relay: {
        enabled: false
      },
      peerDiscovery: {
        autoDial: false
      }
    }
  })

  const dialProtocol = node.dialProtocol.bind(node)

  node.dialProtocol = async (peer: PeerId | Multiaddr, protocols: string[], options: any) => {
    if (PeerId.isPeerId(peer)) {
      return dialProtocol(peer, protocols, options)
    }

    return dialProtocol(maDestination, protocols, options)
  }

  node.handle(TEST_PROTOCOL, async ({ stream }) => {
    await pipe(stream.source, stream.sink)
  })

  await node.start()

  return node
}

function getPeerStore() {
  const peerStore = new Set<Multiaddr>()

  return {
    add: peerStore.add.bind(peerStore),
    get: () => {
      // Make sure that Typescript does not build unit test if Libp2p API changes.
      const multiaddrs: Pick<ReturnType<Libp2p['peerStore']['get']>, 'addresses'>['addresses'] = []
      for (const value of peerStore.values()) {
        multiaddrs.push({
          multiaddr: value,
          isCertified: true
        })
      }

      // Libp2p's return value has more properties but
      // dialHelper only uses the Multiaddresses
      return { addresses: multiaddrs }
    }
  }
}

describe('test dialHelper', function () {
  it('call non-existing', async function () {
    const peerA = await getNode(Alice)

    const result = await dialHelper(peerA, Bob, TEST_PROTOCOL)

    assert(result.status === DialStatus.DIAL_ERROR)
    assert(result.dhtContacted == false)

    // Shutdown node
    await peerA.stop()
  })

  it('regular dial', async function () {
    const peerA = await getNode(Alice)
    const peerB = await getNode(Bob)

    peerA.peerStore.addressBook.add(peerB.peerId, peerB.multiaddrs)

    const result = await dialHelper(peerA, Bob, TEST_PROTOCOL)

    assert(result.status === DialStatus.SUCCESS)

    pipe(TEST_MESSAGE, result.resp.stream.sink)

    for await (const msg of result.resp.stream.source) {
      assert(u8aEquals(msg.slice(), TEST_MESSAGE))
    }

    // Shutdown nodes
    await Promise.all([peerA.stop(), peerB.stop()])
  })

  it('call non-existing with DHT', async function () {
    const peerA = await getNode(Alice, true)

    const result = await dialHelper(peerA, Bob, TEST_PROTOCOL)

    assert(result.status === DialStatus.DHT_ERROR, `Must return dht error`)

    // Shutdown node
    await peerA.stop()
  })

  it('regular dial with DHT', async function () {
    const peerB = await getNode(Bob, true)
    const peerC = await getNode(Chris, true)

    // Secretly tell peerA the address of peerC
    const peerA = await getNode(Alice, true, peerC.multiaddrs[0])

    peerB.peerStore.addressBook.add(peerA.peerId, peerA.multiaddrs)
    peerA.peerStore.addressBook.add(peerB.peerId, peerB.multiaddrs)

    peerB.peerStore.addressBook.add(peerC.peerId, peerC.multiaddrs)
    peerC.peerStore.addressBook.add(peerB.peerId, peerB.multiaddrs)

    await peerA.start()
    await peerB.start()
    await peerC.start()

    await peerA.dial(peerB.peerId)
    await peerC.dial(peerB.peerId)

    await new Promise((resolve) => setTimeout(resolve, 200))

    await peerB.contentRouting.provide(await createRelayerKey(Chris))

    await new Promise((resolve) => setTimeout(resolve, 200))

    let result = await dialHelper(peerA, Chris, TEST_PROTOCOL)

    assert(result.status === DialStatus.SUCCESS, `Dial must be successful`)

    pipe(TEST_MESSAGE, result.resp.stream.sink)

    for await (const msg of result.resp.stream.source) {
      assert(u8aEquals(msg.slice(), TEST_MESSAGE))
    }

    // Shutdown nodes
    await Promise.all([peerA.stop(), peerB.stop(), peerC.stop()])
  })

  it('DHT does not find any new addresses', async function () {
    const peerStore = getPeerStore()
    const peerA = {
      contentRouting: {
        // Non-empty array
        routers: [undefined],
        // Returning an empty iterator
        findProviders: () => (async function* () {})()
      },
      dialProtocol: () => Promise.resolve(undefined),
      peerStore
    }

    peerStore.add(new Multiaddr(`/ip4/127.0.0.1/tcp/123/p2p/${Bob.toB58String()}`))

    // Try to call Bob but does not exist
    const result = await dialHelper(peerA, Bob, TEST_PROTOCOL)

    // Must fail with a DHT error because we obviously can't find
    // Bob's relay address in the DHT
    assert(result.status === DialStatus.DHT_ERROR)
  })

  it('DHT throws an error', async function () {
    const peerStore = getPeerStore()
    const peerA = {
      contentRouting: {
        // Non-empty array
        routers: [undefined],
        // Returning an empty iterator
        findProviders: () =>
          (async function* () {
            throw Error(`boom`)
          })()
      },
      dialProtocol: () => Promise.resolve(undefined),
      peerStore
    }

    peerStore.add(new Multiaddr(`/ip4/127.0.0.1/tcp/123/p2p/${Bob.toB58String()}`))

    const result = await dialHelper(peerA, Bob, TEST_PROTOCOL)

    assert(result.status === DialStatus.DHT_ERROR)
  })
})
