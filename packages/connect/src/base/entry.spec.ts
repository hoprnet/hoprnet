import { EntryNodes, RELAY_CHANGED_EVENT } from './entry.js'
import { getPeerStoreEntry } from './utils.spec.js'
import { CAN_RELAY_PROTOCOLS, OK } from '../constants.js'
import type { PeerStoreType, PublicNodesEmitter, Stream } from '../types.js'
import {} from '@libp2p/peer-store'

import assert from 'assert'
import { once, EventEmitter } from 'events'

import type { PeerId } from '@libp2p/interface-peer-id'
import { Multiaddr } from '@multiformats/multiaddr'

import { privKeyToPeerId, defer, createCircuitAddress } from '@hoprnet/hopr-utils'
import { peerIdFromString } from '@libp2p/peer-id'
import { createFakeComponents, createFakeNetwork, connectEvent } from '../utils/libp2p.mock.spec.js'

async function handleDefaultStream(stream: Stream, throwError: boolean = false) {
  stream.sink(
    (async function* () {
      if (throwError) {
        throw Error(`boom - protocol error`)
      } else {
        yield OK
      }
    })()
  )
  for await (const _msg of stream.source) {
  }
}
/**
 * Decorated EntryNodes class that allows direct access
 * to protected class elements
 */
class TestingEntryNodes extends EntryNodes {
  // @ts-ignore
  public availableEntryNodes: InstanceType<typeof EntryNodes>['availableEntryNodes']

  // @ts-ignore
  public offlineEntryNodes: InstanceType<typeof EntryNodes>['offlineEntryNodes']
  // @ts-ignore
  public usedRelays: InstanceType<typeof EntryNodes>['usedRelays']

  // @ts-ignore
  public uncheckedEntryNodes: InstanceType<typeof EntryNodes>['uncheckedEntryNodes']

  public onNewRelay(...args: Parameters<InstanceType<typeof EntryNodes>['onNewRelay']>) {
    return super.onNewRelay(...args)
  }
  public onRemoveRelay(peer: PeerId) {
    return super.onRemoveRelay(peer)
  }

  public updateRecords(...args: Parameters<InstanceType<typeof EntryNodes>['updateRecords']>) {
    return super.updateRecords(...args)
  }

  public async updatePublicNodes() {
    return super.updatePublicNodes()
  }
}

const peerId = peerIdFromString(`16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit`)
const secondPeer = peerIdFromString(`16Uiu2HAmLpqczAGfgmJchVgVk233rmB2T3DSn2gPG6JMa5brEHZ1`)

describe('entry node functionality - basic functionality', function () {
  it('add public nodes', async function () {
    const network = createFakeNetwork()
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes(
      {},
      {
        maxRelaysPerNode,
        minRelaysPerNode: maxRelaysPerNode
      }
    )

    entryNodes.init(await createFakeComponents(peerId, network))
    entryNodes.start()

    const peerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/0`)

    // Don't contact any nodes
    entryNodes.usedRelays = Array.from({ length: maxRelaysPerNode }) as any

    entryNodes.onNewRelay(peerStoreEntry)
    // Should filter duplicate
    entryNodes.onNewRelay(peerStoreEntry)

    const uncheckedNodes = entryNodes.getUncheckedEntryNodes()

    assert(uncheckedNodes.length == 1, `Unchecked nodes must contain one entry`)
    assert(uncheckedNodes[0].id.equals(peerStoreEntry.id), `id must match the generated one`)
    assert(uncheckedNodes[0].multiaddrs.length == peerStoreEntry.multiaddrs.length, `must not contain more multiaddrs`)

    entryNodes.stop()
    network.stop()
  })

  it('remove an offline node', function () {
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes(
      {},
      {
        maxRelaysPerNode,
        minRelaysPerNode: maxRelaysPerNode
      }
    )

    entryNodes.start()

    const peerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/0`)

    entryNodes.availableEntryNodes.push({
      ...peerStoreEntry,
      latency: 23
    })

    // Don't contact any nodes
    entryNodes.usedRelays = Array.from({ length: maxRelaysPerNode }) as any

    entryNodes.onRemoveRelay(peerStoreEntry.id)

    const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
    assert(availablePublicNodes.length == 0, `must remove node from public nodes`)

    entryNodes.stop()
  })

  it('update existing unchecked nodes', async function () {
    const network = createFakeNetwork()
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes(
      {},
      {
        maxRelaysPerNode,
        minRelaysPerNode: maxRelaysPerNode
      }
    )

    entryNodes.init(await createFakeComponents(peerId, network))
    entryNodes.start()

    const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/123`, secondPeer)
    const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/456`, secondPeer)

    // Don't contact any nodes
    entryNodes.usedRelays = Array.from({ length: maxRelaysPerNode }) as any
    entryNodes.onNewRelay(firstPeerStoreEntry)
    // Should filter duplicate
    entryNodes.onNewRelay(secondPeerStoreEntry)

    assert(entryNodes.uncheckedEntryNodes.length == 1)
    assert(entryNodes.uncheckedEntryNodes[0].multiaddrs.length == 2)
    network.stop()
  })

  it('update addresses of available public nodes', async function () {
    const network = createFakeNetwork()
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(await createFakeComponents(peerId, network))
    entryNodes.start()

    const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/123`, secondPeer)
    const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/456`, secondPeer)

    // Don't contact any nodes
    entryNodes.usedRelays = Array.from({ length: maxRelaysPerNode }) as any

    entryNodes.availableEntryNodes.push({
      id: secondPeer,
      multiaddrs: [],
      latency: 23
    })

    entryNodes.onNewRelay(firstPeerStoreEntry)
    entryNodes.onNewRelay(secondPeerStoreEntry)

    assert(entryNodes.uncheckedEntryNodes.length == 0, `Unchecked nodes must not contain any entry`)
    assert(entryNodes.availableEntryNodes.length == 1, `must not contain more multiaddrs`)
    assert(entryNodes.availableEntryNodes[0].multiaddrs.length == 2)

    entryNodes.stop()
    network.stop()
  })

  it('update addresses of offline public nodes', async function () {
    const network = createFakeNetwork()
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes(
      {},
      {
        maxRelaysPerNode,
        minRelaysPerNode: maxRelaysPerNode
      }
    )

    entryNodes.init(await createFakeComponents(peerId, network))
    entryNodes.start()

    const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/123`, secondPeer)
    const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/456`, secondPeer)

    // Don't contact any nodes
    entryNodes.usedRelays = Array.from({ length: maxRelaysPerNode }) as any

    entryNodes.offlineEntryNodes.push({
      id: secondPeer,
      multiaddrs: []
    })

    entryNodes.onNewRelay(firstPeerStoreEntry)
    entryNodes.onNewRelay(secondPeerStoreEntry)

    assert(entryNodes.uncheckedEntryNodes.length == 0, `Unchecked nodes must not contain any entry`)
    assert(entryNodes.offlineEntryNodes.length == 1, `must not contain more multiaddrs`)
    assert(entryNodes.offlineEntryNodes[0].multiaddrs.length == 2)

    entryNodes.stop()
    network.stop()
  })
})

describe('entry node functionality', function () {
  it('contact potential relays and update relay addresses', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/12345`)

    await createFakeComponents(relay.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: relay.multiaddrs
    })

    const entryNodes = new TestingEntryNodes({ initialNodes: [relay] })

    entryNodes.init(await createFakeComponents(peerId, network))

    // activate NAT functionalities
    entryNodes.enable()

    // Automatically contacts entryNodes (as part of Node startup)
    await entryNodes.afterStart()

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 1, `must contain exactly one public node`)
    assert(availableEntryNodes[0].id.equals(relay.id), `must contain correct peerId`)
    assert(availableEntryNodes[0].latency >= 0, `latency must be non-negative`)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays != undefined, `must expose relay addrs`)
    assert(usedRelays.length == 1, `must expose exactly one relay addrs`)
    assert(usedRelays[0].equals(createCircuitAddress(relay.id)), `must expose the right relay address`)

    const usedRelayPeerIds = entryNodes.getUsedRelayPeerIds()
    assert(usedRelayPeerIds != undefined, `must expose at least one peerId`)
    assert(usedRelayPeerIds.length == 1, `must expose exactly one peerId`)
    assert(usedRelayPeerIds[0].equals(relay.id))

    network.stop()
    entryNodes.stop()
  })

  it('respond with positive latencies, negative latencies, errors and undefined', async function () {
    // Should be all different from each other
    const network = createFakeNetwork()
    const Alice = privKeyToPeerId('0xa544c6684d500b63f96bb6b4196b90a77e71da74f481578fb6e952422189f2bb')
    const Bob = privKeyToPeerId('0xbfdd91247bc19340fe6fc5e91358372ae15cc39a377e163167cfee3f48264fa1')
    const Chris = privKeyToPeerId('0xb0f7016efb37ecefedd7f26274870701adc607320e7ca4467af35ae35470e4ce')
    const Dave = privKeyToPeerId('0x935c28ba604be4912996e4652e7df5bf49f4c3bb5016ebb4c46c3b4575e3c412')

    const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`, Alice)
    const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`, Bob)
    const thirdPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/3`, Chris)
    const fourthPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/4`, Dave)

    await createFakeComponents(firstPeerStoreEntry.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: firstPeerStoreEntry.multiaddrs
    })
    await createFakeComponents(secondPeerStoreEntry.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: secondPeerStoreEntry.multiaddrs
    })

    const entryNodeContactTimeout = 1e3

    const entryNodes = new TestingEntryNodes(
      { initialNodes: [] },
      {
        maxRelaysPerNode: 1,
        minRelaysPerNode: 1,
        maxParallelDials: 1,
        contactTimeout: entryNodeContactTimeout
      }
    )

    entryNodes.uncheckedEntryNodes.push(
      fourthPeerStoreEntry,
      thirdPeerStoreEntry,
      secondPeerStoreEntry,
      firstPeerStoreEntry
    )

    entryNodes.init(
      await createFakeComponents(peerId, network, {
        outerDial: (self: PeerId, ma: Multiaddr) => {
          switch (ma.toString()) {
            case firstPeerStoreEntry.multiaddrs[0].toString():
              return network.connect(self, ma)
            case secondPeerStoreEntry.multiaddrs[0].toString():
              return network.connect(self, ma)
            case fourthPeerStoreEntry.multiaddrs[0].toString():
              return network.connect(self, ma, true)
            default:
              throw Error(`boom - connection error`)
          }
        }
      })
    )

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    assert(entryNodes.getUsedRelayPeerIds().length == 1)
    assert(entryNodes.getUsedRelayPeerIds()[0].equals(Bob))

    assert(entryNodes.uncheckedEntryNodes.length == 1)
    assert(entryNodes.uncheckedEntryNodes[0].id.equals(Alice))

    assert(entryNodes.offlineEntryNodes.length == 2)
    assert(entryNodes.offlineEntryNodes.some((node) => node.id.equals(Chris)))
    assert(entryNodes.offlineEntryNodes.some((node) => node.id.equals(Dave)))

    entryNodes.stop()
    network.stop()
  })

  it('expose limited number of relay addresses', async function () {
    const network = createFakeNetwork()

    const maxParallelDials = 3
    const maxRelaysPerNode = maxParallelDials + 1

    const relayNodes = await Promise.all(
      Array.from<undefined, Promise<PeerStoreType>>(
        { length: maxRelaysPerNode },
        async (_value: undefined, index: number) => {
          const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${index}`)

          await createFakeComponents(relay.id, network, {
            protocols: [
              [
                CAN_RELAY_PROTOCOLS(),
                ({ stream }) => {
                  return handleDefaultStream(stream)
                }
              ]
            ],
            listeningAddrs: relay.multiaddrs
          })

          return relay
        }
      )
    )

    const additionalOfflineNodes = [getPeerStoreEntry(`/ip4/127.0.0.1/tcp/23`)]

    const entryNodes = new TestingEntryNodes(
      {
        initialNodes: relayNodes.concat(additionalOfflineNodes)
      },
      {
        maxParallelDials
      }
    )

    entryNodes.init(await createFakeComponents(peerId, network))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays != undefined, `must expose relay addresses`)
    assert(usedRelays.length == maxRelaysPerNode, `must expose ${maxRelaysPerNode} relay addresses`)

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == maxParallelDials + 1)
    assert(
      relayNodes.every((relayNode) =>
        availableEntryNodes.some((availableEntryNode) => availableEntryNode.id.equals(relayNode.id))
      ),
      `must contain all relay nodes`
    )

    // cleanup
    network.stop()
    entryNodes.stop()
  })

  it('update nodes once node became offline', async function () {
    const network = createFakeNetwork()

    const newNode = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)
    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`)

    await createFakeComponents(newNode.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: newNode.multiaddrs
    })

    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(await createFakeComponents(peerId, network))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()
    entryNodes.uncheckedEntryNodes.push(newNode)

    let usedRelay = {
      relayDirectAddress: new Multiaddr('/ip4/127.0.0.1/tcp/1234'),
      ourCircuitAddress: new Multiaddr(`/p2p/${relay.id.toString()}/p2p-circuit`)
    }

    entryNodes.usedRelays.push(usedRelay)

    // Should have one unchecked node and one relay node
    assert(entryNodes.getUsedRelayAddresses().length == 1)
    assert(entryNodes.getUncheckedEntryNodes().length == 1)

    const updatePromise = once(entryNodes, RELAY_CHANGED_EVENT)

    await entryNodes.onRemoveRelay(relay.id)

    await updatePromise

    assert(entryNodes.getAvailabeEntryNodes().length == 1)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(entryNodes.getUsedRelayAddresses().length == 1)

    assert(usedRelays[0].equals(createCircuitAddress(newNode.id)))

    network.stop()
    entryNodes.stop()
  })

  it('take those nodes that are online', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    await createFakeComponents(relay.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: relay.multiaddrs
    })

    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(await createFakeComponents(peerId, network))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    const fakeNode = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`)

    entryNodes.uncheckedEntryNodes.push(relay)
    entryNodes.uncheckedEntryNodes.push(fakeNode)

    await entryNodes.updatePublicNodes()

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 1)
    assert(availableEntryNodes[0].id.equals(relay.id))

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 1)
    assert(usedRelays[0].equals(createCircuitAddress(relay.id)))

    network.stop()
    entryNodes.stop()
  })

  it('no available entry nodes', async function () {
    const network = createFakeNetwork()

    const offlineRelay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(await createFakeComponents(peerId, network))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    entryNodes.uncheckedEntryNodes.push(offlineRelay)

    await entryNodes.updatePublicNodes()

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 0)

    network.stop()
    entryNodes.stop()
  })

  it('do not emit listening event if nothing has changed', async function () {
    const network = createFakeNetwork()
    const entryNodes = new TestingEntryNodes({})

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    await createFakeComponents(relay.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: relay.multiaddrs
    })

    entryNodes.init(await createFakeComponents(peerId, network))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    let usedRelay = {
      relayDirectAddress: new Multiaddr(`/ip4/127.0.0.1/tcp/1`),
      ourCircuitAddress: new Multiaddr(`/p2p/${relay.id.toString()}/p2p-circuit`)
    }

    entryNodes.availableEntryNodes.push({ ...relay, latency: 23 })
    entryNodes.usedRelays.push(usedRelay)

    entryNodes.once('listening', () =>
      assert.fail(`must not throw listening event if list of entry nodes has not changed`)
    )

    await entryNodes.updatePublicNodes()

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 1)
    assert(availableEntryNodes[0].id.equals(relay.id))

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 1)
    assert(usedRelays[0].equals(createCircuitAddress(relay.id)))

    entryNodes.stop()
    network.stop()
  })

  // @TODO to be fixed
  // it('do not contact nodes we are already connected to', async function () {
  //   const entryNodes = new TestingEntryNodes({})

  //   entryNodes.init(createFakeComponents(peerId))

  //   const ma = new Multiaddr('/ip4/8.8.8.8/tcp/9091')

  //   const peerStoreEntry = getPeerStoreEntry(ma.toString())

  //   entryNodes.usedRelays.push({
  //     relayDirectAddress: ma,
  //     ourCircuitAddress: new Multiaddr(`/p2p/${peerStoreEntry.id.toString()}/p2p-circuit/p2p/${peerId.toString()}`)
  //   })

  //   // activate NAT functionalities
  //   entryNodes.enable()

  //   await entryNodes.afterStart()

  //   await entryNodes.onNewRelay(peerStoreEntry)

  //   const uncheckedNodes = entryNodes.getUncheckedEntryNodes()

  //   assert(uncheckedNodes.length == 0, `Unchecked nodes must be gone`)

  //   const usedRelays = entryNodes.getUsedRelayAddresses()
  //   assert(usedRelays.length == 0, `must not expose any relay addrs`)
  //   assert(usedRelays[0].equals(createCircuitAddress(peerStoreEntry.id)))

  //   entryNodes.stop()
  // })
})

describe('entry node functionality - event propagation', function () {
  it('events should trigger actions', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    await createFakeComponents(relay.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: relay.multiaddrs
    })

    const publicNodes = new EventEmitter() as PublicNodesEmitter
    const entryNodes = new TestingEntryNodes(
      {
        publicNodes
      },
      {
        contactTimeout: 5
      }
    )

    entryNodes.init(await createFakeComponents(peerId, network))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    publicNodes.emit('addPublicNode', relay)

    await once(entryNodes, RELAY_CHANGED_EVENT)

    network.close(relay.multiaddrs[0], false)

    // "Shutdown" network connection to node
    publicNodes.emit('removePublicNode', relay.id)

    await once(entryNodes, RELAY_CHANGED_EVENT)

    console.log(`before second event`)

    entryNodes.once(RELAY_CHANGED_EVENT, () => assert.fail('Must not throw the relay:changed event'))

    await entryNodes.updatePublicNodes()

    entryNodes.stop()
    network.stop()
  })
})

describe('entry node functionality - dht functionality', function () {
  it('renew DHT entry', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    let connectAttempts = 0
    await createFakeComponents(relay.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            connectAttempts++
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: relay.multiaddrs
    })

    const publicNodes: PublicNodesEmitter = new EventEmitter()

    const CUSTOM_DHT_RENEWAL_TIMEOUT = 100 // very short timeout

    const entryNodes = new TestingEntryNodes(
      {
        dhtRenewalTimeout: CUSTOM_DHT_RENEWAL_TIMEOUT,
        publicNodes
      },
      {
        minRelaysPerNode: 1,
        contactTimeout: CUSTOM_DHT_RENEWAL_TIMEOUT / 2
      }
    )

    entryNodes.init(await createFakeComponents(peerId, network))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    publicNodes.emit('addPublicNode', relay)

    await new Promise((resolve) => setTimeout(resolve, 1e3))

    // depends on scheduler
    assert([9, 10].includes(connectAttempts), `Should capture at least 9 renews but not more than 10`)

    network.close(relay.multiaddrs[0], false)
    entryNodes.stop()
    network.stop()
  })
})

describe('entry node functionality - automatic reconnect', function () {
  it('reconnect on disconnect - temporarily offline', async function () {
    const network = createFakeNetwork()
    const relay = getPeerStoreEntry(`/ip4/1.2.3.4/tcp/1`)
    await createFakeComponents(relay.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: relay.multiaddrs
    })
    let secondAttempt = defer<void>()
    let connectAttempt = 0
    const entryNodes = new TestingEntryNodes(
      // Should be successful after second try
      {
        entryNodeReconnectBaseTimeout: 1,
        entryNodeReconnectBackoff: 5
      }
    )

    entryNodes.init(
      await createFakeComponents(peerId, network, {
        outerDial: (self: PeerId, ma: Multiaddr, opts: any) => {
          switch (connectAttempt++) {
            case 0:
              return network.connect(self, ma, opts)
            case 1:
              throw Error(`boom`)
            case 2:
              secondAttempt.resolve()
              return network.connect(self, ma, opts)
            default:
              throw Error(`boom`)
          }
        }
      })
    )

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    const updated = once(entryNodes, RELAY_CHANGED_EVENT)
    entryNodes.onNewRelay(relay)
    await updated

    // Should eventually remove relay from list
    // intermediate solution
    network.close(relay.multiaddrs[0])
    network.events.removeAllListeners(connectEvent(relay.multiaddrs[0]))

    await secondAttempt.promise

    // Wait for end of event loop
    await new Promise((resolve) => setImmediate(resolve))

    const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
    assert(availablePublicNodes.length == 1, `must keep entry node after reconnect`)
    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 1, `must keep relay address after reconnect`)

    network.stop()
    entryNodes.stop()
  })

  it('reconnect on disconnect - permanently offline', async function () {
    const network = createFakeNetwork()
    const relay = getPeerStoreEntry(`/ip4/1.2.3.4/tcp/1`)

    await createFakeComponents(relay.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: relay.multiaddrs
    })
    let connectAttempt = 0
    const entryNodes = new TestingEntryNodes({
      entryNodeReconnectBaseTimeout: 1,
      entryNodeReconnectBackoff: 5
    })

    entryNodes.init(
      await createFakeComponents(peerId, network, {
        outerDial: (self: PeerId, ma: Multiaddr, opts: any) => {
          switch (connectAttempt++) {
            case 0:
              return network.connect(self, ma, opts)
            default:
              throw Error(`boom - connection error`)
          }
        }
      })
    )

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    const entryNodeAdded = once(entryNodes, RELAY_CHANGED_EVENT)

    // Add entry node
    entryNodes.onNewRelay(relay)
    await entryNodeAdded

    const entryNodeRemoved = once(entryNodes, RELAY_CHANGED_EVENT)

    // "Shutdown" node
    network.events.removeAllListeners(connectEvent(relay.multiaddrs[0]))
    network.close(relay.multiaddrs[0])

    await entryNodeRemoved

    const availablePublicNodes = entryNodes.getAvailabeEntryNodes()

    assert(availablePublicNodes.length == 0, `must remove node from public nodes`)

    assert(entryNodes.getUsedRelayAddresses().length == 0, `must not expose any relay addrs`)

    network.stop()
    entryNodes.stop()
  })
})

describe('entry node functionality - min relays per node', function () {
  it('respect minRelayPerNode on peer offline event', async function () {
    const network = createFakeNetwork()

    const Alice = privKeyToPeerId('0xa544c6684d500b63f96bb6b4196b90a77e71da74f481578fb6e952422189f2bb')
    const Bob = privKeyToPeerId('0xbfdd91247bc19340fe6fc5e91358372ae15cc39a377e163167cfee3f48264fa1')

    const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`, Alice)
    const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`, Bob)

    await createFakeComponents(firstPeerStoreEntry.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: firstPeerStoreEntry.multiaddrs
    })
    await createFakeComponents(secondPeerStoreEntry.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: secondPeerStoreEntry.multiaddrs
    })

    let connectionAttempt = false

    const entryNodes = new TestingEntryNodes(
      // Should fail after second try
      {},
      { maxParallelDials: 5, maxRelaysPerNode: 2, minRelaysPerNode: 0 }
    )

    entryNodes.init(
      await createFakeComponents(peerId, network, {
        outerDial: (self: PeerId, ma: Multiaddr, opts: any) => {
          connectionAttempt = true
          return network.connect(self, ma, opts)
        }
      })
    )

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    entryNodes.availableEntryNodes.push(
      { ...firstPeerStoreEntry, latency: 23 },
      { ...secondPeerStoreEntry, latency: 24 }
    )

    entryNodes.usedRelays.push(
      {
        ourCircuitAddress: createCircuitAddress(Alice),
        relayDirectAddress: firstPeerStoreEntry.multiaddrs[0]
      },
      {
        ourCircuitAddress: createCircuitAddress(Bob),
        relayDirectAddress: secondPeerStoreEntry.multiaddrs[0]
      }
    )

    const peerRemovedPromise = once(entryNodes, RELAY_CHANGED_EVENT)
    entryNodes.onRemoveRelay(Alice)

    await peerRemovedPromise

    if (connectionAttempt) {
      assert.fail(`Should not contact any node`)
    }

    assert(entryNodes.usedRelays.length == 1)
    assert(entryNodes.getUsedRelayPeerIds()[0].equals(Bob))

    assert(entryNodes.availableEntryNodes.length == 1)
    assert(entryNodes.availableEntryNodes[0].id.equals(Bob))

    entryNodes.stop()
    network.stop()
  })

  it('respect minRelayPerNode on entry node disconnect', async function () {
    const network = createFakeNetwork()
    const Alice = privKeyToPeerId('0xa544c6684d500b63f96bb6b4196b90a77e71da74f481578fb6e952422189f2bb')
    const Bob = privKeyToPeerId('0xbfdd91247bc19340fe6fc5e91358372ae15cc39a377e163167cfee3f48264fa1')

    const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`, Alice)
    const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`, Bob)

    await createFakeComponents(firstPeerStoreEntry.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: firstPeerStoreEntry.multiaddrs
    })
    await createFakeComponents(secondPeerStoreEntry.id, network, {
      protocols: [
        [
          CAN_RELAY_PROTOCOLS(),
          ({ stream }) => {
            return handleDefaultStream(stream)
          }
        ]
      ],
      listeningAddrs: secondPeerStoreEntry.multiaddrs
    })

    let connectedMoreThanOnce = false
    const connectionAttempts = new Map<string, number>()
    // let secondAttempt = defer<void>()
    const entryNodes = new TestingEntryNodes(
      // Should be successful after second try
      {
        entryNodeReconnectBaseTimeout: 2,
        entryNodeReconnectBackoff: 1000
      },
      {
        maxRelaysPerNode: 2,
        minRelaysPerNode: 0
      }
    )

    entryNodes.init(
      await createFakeComponents(peerId, network, {
        outerDial: ((self: PeerId, ma: Multiaddr) => {
          const connectionAttempt = connectionAttempts.get(ma.getPeerId() as string)

          // Allow 2 reconnect attempt but no additional attempt
          if (connectionAttempt == undefined) {
            connectionAttempts.set(ma.getPeerId() as string, 1)
            return network.connect(self, ma)
          } else if (connectionAttempt == 1) {
            connectionAttempts.set(ma.getPeerId() as string, 2)
          } else if (connectionAttempt == 2) {
            connectionAttempts.set(ma.getPeerId() as string, 3)
          } else {
            connectedMoreThanOnce = true
          }
        }) as any
      })
    )

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    entryNodes.availableEntryNodes.push(
      { ...firstPeerStoreEntry, latency: 23 },
      { ...secondPeerStoreEntry, latency: 24 }
    )

    const relayListUpdated = once(entryNodes, RELAY_CHANGED_EVENT)

    await entryNodes.updatePublicNodes()

    // entryNodes.onNewRelay(relay)
    await relayListUpdated

    const secondRelayListUpdate = once(entryNodes, RELAY_CHANGED_EVENT)

    // intermediate solution
    network.close(firstPeerStoreEntry.multiaddrs[0])
    network.events.removeAllListeners(connectEvent(firstPeerStoreEntry.multiaddrs[0]))

    await secondRelayListUpdate

    if (connectedMoreThanOnce) {
      assert.fail(`Must not connect more than once`)
    }

    const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
    assert(availablePublicNodes.length == 1, `must keep entry node after reconnect`)
    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 1, `must keep relay address after reconnect`)

    network.stop()
    entryNodes.stop()
  })
})
