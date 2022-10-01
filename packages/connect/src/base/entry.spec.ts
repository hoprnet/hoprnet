import { EntryNodes, RELAY_CHANGED_EVENT } from './entry.js'
import { createPeerId, getPeerStoreEntry } from './utils.spec.js'
import { CODE_P2P, OK } from '../constants.js'
import type { PeerStoreType, PublicNodesEmitter } from '../types.js'
import {} from '@libp2p/peer-store'

import assert from 'assert'
import { once, EventEmitter } from 'events'

import { type PeerId, isPeerId } from '@libp2p/interface-peer-id'
import type { DialOptions } from '@libp2p/interface-transport'
import { multiaddr, Multiaddr, protocols } from '@multiformats/multiaddr'
import type { Connection, ProtocolStream } from '@libp2p/interface-connection'
import type { Dialer, ConnectionManagerEvents } from '@libp2p/interface-connection-manager'
import type { Components } from '@libp2p/interfaces/components'
import type { AbortOptions } from '@libp2p/interfaces'
import { privKeyToPeerId, defer, createCircuitAddress } from '@hoprnet/hopr-utils'
import { peerIdFromString } from '@libp2p/peer-id'
import { PeerStore } from '@libp2p/interfaces/peer-store'
import { CustomEvent, EventEmitter as TypedEventEmitter } from '@libp2p/interfaces/events'

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

class MyConnectionManager extends TypedEventEmitter<ConnectionManagerEvents> {
  dialer: Dialer

  constructor(peerStore: PeerStore, outerDial?: Dialer['dial']) {
    super()

    this.dialer = {
      async dial(peer: PeerId | Multiaddr, opts: Parameters<Dialer['dial']>[1]) {
        if (outerDial == undefined) {
          return fakeConnection()
        }

        if (isPeerId(peer)) {
          const addrs = await peerStore.addressBook.get(peer)

          if (addrs == null || addrs.length == 0) {
            throw Error(`No addresses known`)
          }

          let conn: Connection | undefined
          for (const addr of addrs) {
            try {
              console.log(addr.multiaddr)
              conn = (await outerDial(
                addr.multiaddr.decapsulateCode(CODE_P2P).encapsulate(`/p2p/${peer.toString()}`),
                opts
              )) as any
            } catch {
              // try next address
              continue
            }
          }

          if (conn == undefined) {
            throw Error(`Dial error: no valid addresses known`)
          }

          return conn as any
        }

        throw Error(`not implemented`)
      }
    } as Dialer
  }

  public getConnections(_peer: PeerId | undefined, _options?: AbortOptions): Connection[] {
    return []
  }
}

function fakePeerStore(): PeerStore {
  const addrs = new Map<string, Multiaddr[]>()

  return {
    addressBook: {
      async get(id: PeerId) {
        return (addrs.get(id.toString()) ?? []).map((ma) => ({ multiaddr: ma, isCertified: true }))
      },
      async add(id: PeerId, multiaddrs: Multiaddr[]) {
        addrs.set(
          id.toString(),
          [
            ...new Set(
              (addrs.get(id.toString()) ?? []).concat(multiaddrs).map((ma) => ma.decapsulateCode(CODE_P2P).toString())
            )
          ].map((str) => new Multiaddr(str))
        )
      }
    }
  } as PeerStore
}

function createFakeComponents(peerId: PeerId, outerDial?: Dialer['dial']) {
  const peerStore = fakePeerStore()

  const connectionManager = new MyConnectionManager(peerStore, outerDial) as Components['connectionManager']

  const getUpgrader = () =>
    ({
      upgradeInbound: (x: any) => x,
      upgradeOutbound: (x: any) => x
    } as Components['upgrader'])

  return {
    getPeerId: () => peerId,
    getConnectionManager: () => connectionManager,
    getUpgrader,
    getPeerStore: () => peerStore
  } as Components
}

function connectEvent(addr: string): string {
  return `connect:${addr}`
}

function disconnectEvent(addr: string) {
  return `disconnect:${addr}`
}

function fakeConnection(throwError: boolean = false): Connection {
  const conn = {
    _closed: false,
    close: async () => {
      // @ts-ignore
      conn._closed = true
    },
    stat: {
      timeline: {
        open: Date.now()
      }
    },
    newStream: (_protocols: string[]) => {
      console.log(`here`, _protocols)
      return Promise.resolve({
        stream: {
          source: (async function* () {
            console.log(`in source`)
            if (throwError) {
              throw Error(`boom - protocol error`)
            } else {
              yield OK
            }
          })() as AsyncIterable<Uint8Array>,
          sink: async (source: AsyncIterableIterator<any>) => {
            // consume the send stream
            for await (const _sth of source) {
            }
          }
        } as ProtocolStream['stream']
      })
    }
  } as unknown as Connection

  return conn as Connection
}

function createFakeNetwork() {
  const network = new EventEmitter()

  const listen = (addr: string) => {
    const emitter = new EventEmitter()
    network.on(connectEvent(addr), () => emitter.emit('connected'))

    return emitter
  }

  const connect: Dialer['dial'] = (async (ma: PeerId | Multiaddr, _opts?: { signal?: AbortSignal }) => {
    const addr = ma.toString()

    if (network.listeners(connectEvent(addr)).length >= 1) {
      network.emit(connectEvent(addr))

      return fakeConnection()
    }

    throw Error(`Cannot connect. Maybe not listening?`)
  }) as any // Multiaddr type clash

  const close = (ma: Multiaddr) => {
    network.emit(disconnectEvent(ma.toString()), ma)
  }

  return {
    events: network,
    listen,
    connect,
    close,
    stop: network.removeAllListeners.bind(network)
  }
}

const peerId = peerIdFromString(`16Uiu2HAm91QFjPepnwjuZWzK5pb5ZS8z8qxQRfKZJNXjkgGNUAit`)
const secondPeer = peerIdFromString(`16Uiu2HAmLpqczAGfgmJchVgVk233rmB2T3DSn2gPG6JMa5brEHZ1`)

describe('entry node functionality - basic functionality', function () {
  it('add public nodes', function () {
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes(
      {},
      {
        maxRelaysPerNode,
        minRelaysPerNode: maxRelaysPerNode
      }
    )

    entryNodes.init(createFakeComponents(peerId))
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

  it('update existing unchecked nodes', function () {
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes(
      {},
      {
        maxRelaysPerNode,
        minRelaysPerNode: maxRelaysPerNode
      }
    )

    entryNodes.init(createFakeComponents(peerId))
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
  })

  it('update addresses of available public nodes', function () {
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(createFakeComponents(peerId))
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
  })

  it('update addresses of offline public nodes', function () {
    const maxRelaysPerNode = 2
    const entryNodes = new TestingEntryNodes(
      {},
      {
        maxRelaysPerNode,
        minRelaysPerNode: maxRelaysPerNode
      }
    )

    entryNodes.init(createFakeComponents(peerId))
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
  })
})

describe.only('entry node functionality', function () {
  it('contact potential relays and update relay addresses', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/12345`)

    const relayListener = network.listen(relay.multiaddrs[0].toString())

    const connectPromise = once(relayListener, 'connected')

    const entryNodes = new TestingEntryNodes({ initialNodes: [relay] })

    entryNodes.init(createFakeComponents(peerId, network.connect))

    // activate NAT functionalities
    entryNodes.enable()

    // Automatically contacts entryNodes (as part of Node startup)
    await entryNodes.afterStart()

    await connectPromise

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

    relayListener.removeAllListeners()
    network.stop()
    entryNodes.stop()
  })

  it('respond with positive latencies, negative latencies, errors and undefined', async function () {
    // Should be all different from each other
    const Alice = privKeyToPeerId('0xa544c6684d500b63f96bb6b4196b90a77e71da74f481578fb6e952422189f2bb')
    const Bob = privKeyToPeerId('0xbfdd91247bc19340fe6fc5e91358372ae15cc39a377e163167cfee3f48264fa1')
    const Chris = privKeyToPeerId('0xb0f7016efb37ecefedd7f26274870701adc607320e7ca4467af35ae35470e4ce')
    const Dave = privKeyToPeerId('0x935c28ba604be4912996e4652e7df5bf49f4c3bb5016ebb4c46c3b4575e3c412')

    const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`, Alice)
    const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`, Bob)
    const thirdPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/3`, Chris)
    const fourthPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/4`, Dave)

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
      createFakeComponents(peerId, (async (ma: PeerId | Multiaddr) => {
        switch (ma.toString()) {
          case firstPeerStoreEntry.multiaddrs[0].toString():
            return fakeConnection()
          case secondPeerStoreEntry.multiaddrs[0].toString():
            return fakeConnection()
          case fourthPeerStoreEntry.multiaddrs[0].toString():
            return fakeConnection(true)
          default:
            throw Error(`boom - connection error`)
        }
      }) as any)
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
  })

  it('expose limited number of relay addresses', async function () {
    const network = createFakeNetwork()

    const maxParallelDials = 3
    const maxRelaysPerNode = maxParallelDials + 1

    const relayNodes = Array.from<undefined, [Promise<any>, PeerStoreType, EventEmitter]>(
      { length: maxRelaysPerNode },
      (_value: undefined, index: number) => {
        const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${index}`)

        const relayListener = network.listen(relay.multiaddrs[0].toString())

        const connectPromise = once(relayListener, 'connected')

        return [connectPromise, relay, relayListener]
      }
    )

    const additionalOfflineNodes = [getPeerStoreEntry(`/ip4/127.0.0.1/tcp/23`)]

    const entryNodes = new TestingEntryNodes(
      {
        initialNodes: relayNodes.map((relayNode) => relayNode[1]).concat(additionalOfflineNodes)
      },
      {
        maxParallelDials
      }
    )

    entryNodes.init(createFakeComponents(peerId, network.connect as any))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    await Promise.all(relayNodes.map((relayNode) => relayNode[0]))

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays != undefined, `must expose relay addresses`)
    assert(usedRelays.length == maxRelaysPerNode, `must expose ${maxRelaysPerNode} relay addresses`)

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == maxParallelDials + 1)
    assert(
      relayNodes.every((relayNode) =>
        availableEntryNodes.some((availableEntryNode) => availableEntryNode.id.equals(relayNode[1].id))
      ),
      `must contain all relay nodes`
    )

    // cleanup
    relayNodes.forEach((relayNode) => relayNode[2].removeAllListeners())
    network.stop()
    entryNodes.stop()
  })

  it('update nodes once node became offline', async function () {
    const network = createFakeNetwork()

    const newNode = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)
    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`)

    const newNodeListener = network.listen(newNode.multiaddrs[0].toString())

    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(createFakeComponents(peerId))

    entryNodes.init(createFakeComponents(peerId, network.connect as any))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()
    entryNodes.uncheckedEntryNodes.push(newNode)

    let usedRelay = {
      relayDirectAddress: new Multiaddr('/ip4/127.0.0.1/tcp/1234'),
      ourCircuitAddress: new Multiaddr(`/p2p/${relay.id.toString()}/p2p-circuit/p2p/${peerId.toString()}`)
    }

    entryNodes.usedRelays.push(usedRelay)

    // Should have one unchecked node and one relay node
    assert(entryNodes.getUsedRelayAddresses().length == 1)
    assert(entryNodes.getUncheckedEntryNodes().length == 1)

    const connectPromise = once(newNodeListener, 'connected')

    const updatePromise = once(entryNodes, RELAY_CHANGED_EVENT)

    await entryNodes.onRemoveRelay(relay.id)

    await Promise.all([connectPromise, updatePromise])

    assert(entryNodes.getAvailabeEntryNodes().length == 1)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(entryNodes.getUsedRelayAddresses().length == 1)

    assert(usedRelays[0].equals(createCircuitAddress(newNode.id)))

    newNodeListener.removeAllListeners()
    network.stop()
    entryNodes.stop()
  })

  it('take those nodes that are online', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)
    const relayListener = network.listen(relay.multiaddrs[0].toString())

    const connectPromise = once(relayListener, 'connected')

    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(createFakeComponents(peerId, network.connect))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    const fakeNode = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`)

    entryNodes.uncheckedEntryNodes.push(relay)
    entryNodes.uncheckedEntryNodes.push(fakeNode)

    await entryNodes.updatePublicNodes()

    await connectPromise

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 1)
    assert(availableEntryNodes[0].id.equals(relay.id))

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 1)
    assert(usedRelays[0].equals(createCircuitAddress(relay.id)))

    network.stop()
    relayListener.removeAllListeners()
    entryNodes.stop()
  })

  it('no available entry nodes', async function () {
    const network = createFakeNetwork()

    const offlineRelay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(createFakeComponents(peerId, network.connect))

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
    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(createFakeComponents(peerId))

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    let usedRelay = {
      relayDirectAddress: new Multiaddr(`/ip4/127.0.0.1/tcp/1`),
      ourCircuitAddress: new Multiaddr(`/p2p/${relay.id.toString()}/p2p-circuit/p2p/${peerId.toString()}`)
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
  })

  it('do not contact nodes we are already connected to', async function () {
    const entryNodes = new TestingEntryNodes({})

    entryNodes.init(createFakeComponents(peerId))

    const ma = new Multiaddr('/ip4/8.8.8.8/tcp/9091')

    const peerStoreEntry = getPeerStoreEntry(ma.toString())

    entryNodes.usedRelays.push({
      relayDirectAddress: ma,
      ourCircuitAddress: new Multiaddr(`/p2p/${peerStoreEntry.id.toString()}/p2p-circuit/p2p/${peerId.toString()}`)
    })

    // activate NAT functionalities
    entryNodes.enable()

    await entryNodes.afterStart()

    await entryNodes.onNewRelay(peerStoreEntry)

    const uncheckedNodes = entryNodes.getUncheckedEntryNodes()

    assert(uncheckedNodes.length == 0, `Unchecked nodes must be gone`)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 1, `must not expose any relay addrs`)
    assert(usedRelays[0].equals(createCircuitAddress(peerStoreEntry.id)))

    entryNodes.stop()
  })
})

// describe('entry node functionality - event propagation', function () {
//   it('events should trigger actions', async function () {
//     const network = createFakeNetwork()

//     const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)
//     const relayListener = network.listen(relay.multiaddrs[0].toString())

//     const publicNodes = new EventEmitter() as PublicNodesEmitter
//     const entryNodes = new TestingEntryNodes(
//       network.connect as any,
//       {
//         publicNodes
//       },
//       {
//         contactTimeout: 5
//       }
//     )

//     entryNodes.init(createFakeComponents(peerId))
//     entryNodes.start()

//     publicNodes.emit('addPublicNode', relay)

//     await once(entryNodes, RELAY_CHANGED_EVENT)

//     network.events.removeAllListeners(connectEvent(relay.multiaddrs[0].toString()))

//     // "Shutdown" network connection to node
//     publicNodes.emit('removePublicNode', relay.id)

//     await once(entryNodes, RELAY_CHANGED_EVENT)

//     entryNodes.once(RELAY_CHANGED_EVENT, () => assert.fail('Must not throw the relay:changed event'))

//     await entryNodes.updatePublicNodes()

//     relayListener.removeAllListeners()
//     entryNodes.stop()
//     network.stop()
//   })
// })

// describe('entry node functionality - dht functionality', function () {
//   it('renew DHT entry', async function () {
//     const network = createFakeNetwork()

//     const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

//     const connectEmitter = network.listen(relay.multiaddrs[0].toString())

//     let renews = 0

//     connectEmitter.on('connected', () => renews++)

//     const publicNodes: PublicNodesEmitter = new EventEmitter()

//     const CUSTOM_DHT_RENEWAL_TIMEOUT = 100 // very short timeout

//     const entryNodes = new TestingEntryNodes(
//       network.connect as any,
//       {
//         dhtRenewalTimeout: CUSTOM_DHT_RENEWAL_TIMEOUT,
//         publicNodes
//       },
//       {
//         minRelaysPerNode: 1,
//         contactTimeout: CUSTOM_DHT_RENEWAL_TIMEOUT / 2
//       }
//     )

//     entryNodes.init(createFakeComponents(peerId))
//     entryNodes.start()

//     publicNodes.emit('addPublicNode', relay)

//     await new Promise((resolve) => setTimeout(resolve, 1e3))

//     // depends on scheduler
//     assert([9, 10].includes(renews), `Should capture at least 9 renews but not more than 10`)

//     connectEmitter.removeAllListeners()
//     entryNodes.stop()
//     network.stop()
//   })
// })

// describe('entry node functionality - automatic reconnect', function () {
//   it('reconnect on disconnect - temporarily offline', async function () {
//     const network = createFakeNetwork()
//     const relay = getPeerStoreEntry(`/ip4/1.2.3.4/tcp/1`)
//     const relayListener = network.listen(relay.multiaddrs[0].toString())
//     let secondAttempt = defer<void>()
//     let connectAttempt = 0
//     const entryNodes = new TestingEntryNodes(
//       (async (ma: Multiaddr, opts: any) => {
//         if (opts.onDisconnect) {
//           network.events.on(disconnectEvent(relay.multiaddrs[0].toString()), opts.onDisconnect)
//         }
//         switch (connectAttempt++) {
//           case 0:
//             return network.connect(ma, opts)
//           case 1:
//             return
//           case 2:
//             secondAttempt.resolve()
//             return network.connect(ma, opts)
//           default:
//             return
//         }
//       }) as any,
//       // Should be successful after second try
//       {
//         entryNodeReconnectBaseTimeout: 1,
//         entryNodeReconnectBackoff: 5
//       }
//     )

//     entryNodes.init(createFakeComponents(peerId))
//     entryNodes.start()

//     const updated = once(entryNodes, RELAY_CHANGED_EVENT)
//     entryNodes.onNewRelay(relay)
//     await updated

//     // Should eventually remove relay from list
//     network.close(relay.multiaddrs[0])
//     await secondAttempt.promise

//     // Wait for end of event loop
//     await new Promise((resolve) => setImmediate(resolve))

//     const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
//     assert(availablePublicNodes.length == 1, `must keep entry node after reconnect`)
//     const usedRelays = entryNodes.getUsedRelayAddresses()
//     assert(usedRelays.length == 1, `must keep relay address after reconnect`)

//     relayListener.removeAllListeners()
//     network.stop()
//     entryNodes.stop()
//   })

//   it('reconnect on disconnect - permanently offline', async function () {
//     const network = createFakeNetwork()
//     const relay = getPeerStoreEntry(`/ip4/1.2.3.4/tcp/1`)
//     const relayListener = network.listen(relay.multiaddrs[0].toString())
//     let connectAttempt = 0
//     const entryNodes = new TestingEntryNodes(
//       (async (ma: Multiaddr, opts: any) => {
//         if (opts.onDisconnect) {
//           network.events.on(disconnectEvent(relay.multiaddrs[0].toString()), opts.onDisconnect)
//         }
//         switch (connectAttempt++) {
//           case 0:
//             return network.connect(ma, opts)
//           default:
//             throw Error(`boom - connection error`)
//         }
//       }) as any,
//       // Should fail after second try
//       {
//         entryNodeReconnectBaseTimeout: 1,
//         entryNodeReconnectBackoff: 5
//       }
//     )

//     entryNodes.init(createFakeComponents(peerId))
//     entryNodes.start()

//     const entryNodeAdded = once(entryNodes, RELAY_CHANGED_EVENT)

//     // Add entry node
//     entryNodes.onNewRelay(relay)
//     await entryNodeAdded

//     const entryNodeRemoved = once(entryNodes, RELAY_CHANGED_EVENT)

//     // "Shutdown" node
//     network.events.removeAllListeners(connectEvent(relay.multiaddrs[0].toString()))
//     network.close(relay.multiaddrs[0])

//     await entryNodeRemoved

//     const availablePublicNodes = entryNodes.getAvailabeEntryNodes()

//     assert(availablePublicNodes.length == 0, `must remove node from public nodes`)

//     assert(entryNodes.getUsedRelayAddresses().length == 0, `must not expose any relay addrs`)

//     relayListener.removeAllListeners()
//     network.stop()
//     entryNodes.stop()
//   })
// })

// describe('entry node functionality - min relays per node', function () {
//   it('respect minRelayPerNode on peer offline event', async function () {
//     const network = createFakeNetwork()

//     const Alice = privKeyToPeerId('0xa544c6684d500b63f96bb6b4196b90a77e71da74f481578fb6e952422189f2bb')
//     const Bob = privKeyToPeerId('0xbfdd91247bc19340fe6fc5e91358372ae15cc39a377e163167cfee3f48264fa1')

//     const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`, Alice)
//     const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`, Bob)

//     let connectionAttempt = false

//     const entryNodes = new TestingEntryNodes(
//       (async (_ma: Multiaddr, _opts: any) => {
//         connectionAttempt = true
//       }) as any,
//       // Should fail after second try
//       {},
//       { maxParallelDials: 5, maxRelaysPerNode: 2, minRelaysPerNode: 0 }
//     )

//     entryNodes.init(createFakeComponents(peerId))
//     entryNodes.start()

//     entryNodes.availableEntryNodes.push(
//       { ...firstPeerStoreEntry, latency: 23 },
//       { ...secondPeerStoreEntry, latency: 24 }
//     )

//     entryNodes.usedRelays.push(
//       {
//         ourCircuitAddress: createCircuitAddress(Alice, peerId),
//         relayDirectAddress: firstPeerStoreEntry.multiaddrs[0]
//       },
//       {
//         ourCircuitAddress: createCircuitAddress(Bob, peerId),
//         relayDirectAddress: secondPeerStoreEntry.multiaddrs[0]
//       }
//     )

//     const peerRemovedPromise = once(entryNodes, RELAY_CHANGED_EVENT)
//     entryNodes.onRemoveRelay(Alice)

//     await peerRemovedPromise

//     if (connectionAttempt) {
//       assert.fail(`Should not contact any node`)
//     }

//     assert(entryNodes.usedRelays.length == 1)
//     assert(entryNodes.getUsedRelayPeerIds()[0].equals(Bob))

//     assert(entryNodes.availableEntryNodes.length == 1)
//     assert(entryNodes.availableEntryNodes[0].id.equals(Bob))

//     entryNodes.stop()
//     network.stop()
//   })

//   it('respect minRelayPerNode on entry node disconnect', async function () {
//     const network = createFakeNetwork()
//     const Alice = privKeyToPeerId('0xa544c6684d500b63f96bb6b4196b90a77e71da74f481578fb6e952422189f2bb')
//     const Bob = privKeyToPeerId('0xbfdd91247bc19340fe6fc5e91358372ae15cc39a377e163167cfee3f48264fa1')

//     const firstPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`, Alice)
//     const secondPeerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`, Bob)

//     const AliceListener = network.listen(firstPeerStoreEntry.multiaddrs[0].toString())
//     const BobListener = network.listen(secondPeerStoreEntry.multiaddrs[0].toString())

//     let connectedMoreThanOnce = false
//     const connectionAttempts = new Map<string, number>()
//     // let secondAttempt = defer<void>()
//     const entryNodes = new TestingEntryNodes(
//       (async (ma: Multiaddr, opts: any) => {
//         if (opts.onDisconnect) {
//           network.events.on(disconnectEvent(ma.toString()), opts.onDisconnect)
//         }
//         const connectionAttempt = connectionAttempts.get(ma.getPeerId() as string)

//         // Allow 2 reconnect attempt but no additional attempt
//         if (connectionAttempt == undefined) {
//           connectionAttempts.set(ma.getPeerId() as string, 1)
//           return network.connect(ma, opts)
//         } else if (connectionAttempt == 1) {
//           connectionAttempts.set(ma.getPeerId() as string, 2)
//         } else if (connectionAttempt == 2) {
//           connectionAttempts.set(ma.getPeerId() as string, 3)
//         } else {
//           connectedMoreThanOnce = true
//         }
//       }) as any,
//       // Should be successful after second try
//       {
//         entryNodeReconnectBaseTimeout: 2,
//         entryNodeReconnectBackoff: 1000
//       },
//       {
//         maxRelaysPerNode: 2,
//         minRelaysPerNode: 0
//       }
//     )

//     entryNodes.init(createFakeComponents(peerId))
//     entryNodes.start()

//     entryNodes.availableEntryNodes.push(
//       { ...firstPeerStoreEntry, latency: 23 },
//       { ...secondPeerStoreEntry, latency: 24 }
//     )

//     const relayListUpdated = once(entryNodes, RELAY_CHANGED_EVENT)

//     await entryNodes.updatePublicNodes()

//     // entryNodes.onNewRelay(relay)
//     await relayListUpdated

//     const secondRelayListUpdate = once(entryNodes, RELAY_CHANGED_EVENT)
//     // // Should eventually remove relay from list
//     network.close(firstPeerStoreEntry.multiaddrs[0])

//     await secondRelayListUpdate

//     if (connectedMoreThanOnce) {
//       assert.fail(`Must not connect more than once`)
//     }

//     const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
//     assert(availablePublicNodes.length == 1, `must keep entry node after reconnect`)
//     const usedRelays = entryNodes.getUsedRelayAddresses()
//     assert(usedRelays.length == 1, `must keep relay address after reconnect`)

//     AliceListener.removeAllListeners()
//     BobListener.removeAllListeners()

//     network.stop()
//     entryNodes.stop()
//   })
// })
