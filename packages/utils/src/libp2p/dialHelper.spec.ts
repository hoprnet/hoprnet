import { NOISE } from '@chainsafe/libp2p-noise'
const MPLEX = require('libp2p-mplex')
import Libp2p from 'libp2p'
import { dial as dialHelper } from './dialHelper'
import TCP from 'libp2p-tcp'
import KadDHT from 'libp2p-kad-dht'
import PeerId from 'peer-id'
import assert from 'assert'
import { Multiaddr } from 'multiaddr'
import pipe from 'it-pipe'
import { u8aEquals } from '../u8a'
import { defer } from '../async/defer'

const TEST_PROTOCOL = '/test'
const TEST_MESSAGE = new TextEncoder().encode('test msg')

async function getNode(id: PeerId, withDHT = false): Promise<Libp2p> {
  const node = await Libp2p.create({
    addresses: {
      listen: [new Multiaddr('/ip4/0.0.0.0/tcp/0').toString()]
    },
    peerId: id,
    modules: {
      transport: [TCP],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE],
      dht: withDHT ? KadDHT : undefined
    },
    config: {
      dht: {
        enabled: withDHT
      },
      nat: {
        enabled: false
      }
    }
  })

  node.handle(TEST_PROTOCOL, async ({ stream }) => {
    await pipe(stream.source, stream.sink)
  })

  await node.start()

  return node
}

describe('test dialHelper', function () {
  let Alice: PeerId
  let Bob: PeerId
  let Chris: PeerId

  before(async function () {
    Alice = await PeerId.create({ keyType: 'secp256k1' })
    Bob = await PeerId.create({ keyType: 'secp256k1' })
    Chris = await PeerId.create({ keyType: 'secp256k1' })

    assert(!Alice.equals(Bob))
    assert(!Alice.equals(Chris))
    assert(!Bob.equals(Chris))
  })

  it('call non-existing', async function () {
    const peerA = await getNode(Alice)

    const result = await dialHelper(peerA, Bob, TEST_PROTOCOL)

    assert(result.status === 'E_DIAL')
    assert(result.dhtContacted == false)

    // Shutdown node
    await peerA.stop()
  })

  it('regular dial', async function () {
    const peerA = await getNode(Alice)
    const peerB = await getNode(Bob)

    peerA.peerStore.addressBook.add(peerB.peerId, peerB.multiaddrs)

    const result = await dialHelper(peerA, Bob, TEST_PROTOCOL)

    assert(result.status === 'SUCCESS')

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

    assert(result.status === 'E_DIAL', `Should print dial error`)
    assert(result.dhtContacted == true, `Should contact DHT`)

    // Shutdown node
    await peerA.stop()
  })

  it('regular dial with DHT', async function () {
    this.timeout(10e3)
    const peerA = await getNode(Alice, true)
    const peerB = await getNode(Bob, true)
    const peerC = await getNode(Chris, true)

    peerB.peerStore.addressBook.add(peerA.peerId, peerA.multiaddrs)
    peerA.peerStore.addressBook.add(peerB.peerId, peerB.multiaddrs)

    peerB.peerStore.addressBook.add(peerC.peerId, peerC.multiaddrs)
    peerC.peerStore.addressBook.add(peerB.peerId, peerB.multiaddrs)

    // Give DHT time to start
    await new Promise((resolve) => setTimeout(resolve, 1500))

    let result = await dialHelper(peerA, Chris, TEST_PROTOCOL)

    // If DHT failed, wait until Alice has found Chris
    if (result.status !== 'SUCCESS') {
      const waitUntilDiscovered = defer<void>()

      peerA.on('peer:discovery', (peerId: PeerId) => {
        if (peerId.equals(Chris)) {
          waitUntilDiscovered.resolve()
        }
      })

      await waitUntilDiscovered.promise

      result = await dialHelper(peerA, Chris, TEST_PROTOCOL)
    }

    assert(result.status === 'SUCCESS', `Dial must be successful`)

    pipe(TEST_MESSAGE, result.resp.stream.sink)

    for await (const msg of result.resp.stream.source) {
      assert(u8aEquals(msg.slice(), TEST_MESSAGE))
    }

    // Shutdown nodes
    await Promise.all([peerA.stop(), peerB.stop(), peerC.stop()])
  })
})
