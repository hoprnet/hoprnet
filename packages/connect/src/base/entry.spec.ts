import { EntryNodes, ENTRY_NODES_MAX_PARALLEL_DIALS, RELAY_CHANGED_EVENT } from './entry'
import type PeerId from 'peer-id'
import assert from 'assert'
import { createPeerId, getPeerStoreEntry } from './utils.spec'
import { once, EventEmitter } from 'events'
import { Multiaddr } from 'multiaddr'
import type Connection from 'libp2p-interfaces/dist/src/connection/connection'

import { MAX_RELAYS_PER_NODE, OK } from '../constants'
import type { HoprConnectDialOptions, PeerStoreType, PublicNodesEmitter } from '../types'
import { defer } from '@hoprnet/hopr-utils'

class FakeConnectionManager {
  public _started: boolean
  private connections: Map<string, Connection[]>

  constructor(started = false) {
    this._started = started
    this.connections = new Map<string, Connection[]>()
  }

  public start() {
    this._started = true
  }

  public getAll(peer: PeerId): Connection[] {
    return this.connections.get(peer.toB58String()) ?? []
  }

  public onDisconnect(_conn: Connection): void {}
}

/**
 * Decorated EntryNodes class that allows direct access
 * to protected class elements
 */
class TestingEntryNodes extends EntryNodes {
  // @ts-ignore
  public availableEntryNodes: InstanceType<typeof EntryNodes>['availableEntryNodes']

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

  public async updatePublicNodes() {
    return super.updatePublicNodes()
  }
}

function connectEvent(addr: string): string {
  return `connect:${addr}`
}

function disconnectEvent(addr: string) {
  return `disconnect:${addr}`
}

function createFakeNetwork() {
  const network = new EventEmitter()

  const listen = (addr: string) => {
    const emitter = new EventEmitter()
    network.on(connectEvent(addr), () => emitter.emit('connected'))

    return emitter
  }

  const connect = (ma: Multiaddr, opts: HoprConnectDialOptions) => {
    const addr = ma.toString()

    if (opts.onDisconnect) {
      network.once(disconnectEvent(addr), () => opts?.onDisconnect?.(ma))
    }

    if (network.listeners(connectEvent(addr)).length >= 1) {
      network.emit(connectEvent(addr))

      const conn = {
        _closed: false,
        close: async () => {
          conn._closed = true
        },
        newStream: (_protocols: string[]) =>
          Promise.resolve({
            stream: {
              source: (async function* () {
                yield OK
              })(),
              sink: async (source: AsyncIterableIterator<any>) => {
                // consume the send stream
                for await (const _sth of source) {
                }
              }
            }
          })
      }
      return Promise.resolve(conn)
    } else {
      return Promise.resolve(undefined)
    }
  }

  const close = (ma: Multiaddr) => {
    network.emit(disconnectEvent(ma.toString()))
  }

  return {
    listen,
    connect,
    close,
    stop: network.removeAllListeners.bind(network)
  }
}

describe('entry node functionality', function () {
  const peerId = createPeerId()
  it('add public nodes', function () {
    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      // Make sure that call is indeed asynchronous
      (async () => new Promise((resolve) => setImmediate(resolve))) as any,
      {}
    )

    entryNodes.start()

    const peerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/0`)

    entryNodes.onNewRelay(peerStoreEntry)
    // Should filter duplicate
    entryNodes.onNewRelay(peerStoreEntry)

    const uncheckedNodes = entryNodes.getUncheckedEntryNodes()

    assert(uncheckedNodes.length == 1, `Unchecked nodes must contain one entry`)
    assert(uncheckedNodes[0].id.equals(peerStoreEntry.id), `id must match the generated one`)
    assert(uncheckedNodes[0].multiaddrs.length == peerStoreEntry.multiaddrs.length, `must not contain more multiaddrs`)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays == undefined || usedRelays.length == 0, `must not expose any internal addrs`)

    entryNodes.stop()
  })

  it('remove an offline node', function () {
    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager()
      },
      (async () => new Promise((resolve) => setImmediate(resolve))) as any,
      {}
    )

    entryNodes.start()

    const peerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/0`)

    entryNodes.availableEntryNodes.push({
      ...peerStoreEntry,
      latency: 23
    })

    entryNodes.onRemoveRelay(peerStoreEntry.id)

    const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
    assert(availablePublicNodes.length == 0, `must remove node from public nodes`)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays == undefined || usedRelays.length == 0, `must not expose any internal addrs`)

    entryNodes.stop()
  })

  it('contact potential relays and update relay addresses', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/12345`)

    const relayListener = network.listen(relay.multiaddrs[0].toString())

    const connectPromise = once(relayListener, 'connected')

    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      network.connect as any,
      {
        initialNodes: [relay]
      }
    )

    entryNodes.start()

    await entryNodes.updatePublicNodes()

    await connectPromise

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 1, `must contain exactly one public node`)
    assert(availableEntryNodes[0].id.equals(relay.id), `must contain correct peerId`)
    assert(availableEntryNodes[0].latency >= 0, `latency must be non-negative`)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays != undefined, `must expose relay addrs`)
    assert(usedRelays.length == 1, `must expose exactly one relay addrs`)
    assert(
      usedRelays[0].toString() === `/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`,
      `must expose the right relay address`
    )

    relayListener.removeAllListeners()
    network.stop()
    entryNodes.stop()
  })

  it('expose limited number of relay addresses', async function () {
    const network = createFakeNetwork()

    const relayNodes = Array.from<undefined, [Promise<any>, PeerStoreType, EventEmitter]>(
      { length: ENTRY_NODES_MAX_PARALLEL_DIALS + 1 },
      (_value: undefined, index: number) => {
        const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${index}`)

        const relayListener = network.listen(relay.multiaddrs[0].toString())

        const connectPromise = once(relayListener, 'connected')

        return [connectPromise, relay, relayListener]
      }
    )

    const additionalOfflineNodes = [getPeerStoreEntry(`/ip4/127.0.0.1/tcp/23`)]

    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      network.connect as any,
      {
        initialNodes: relayNodes.map((relayNode) => relayNode[1]).concat(additionalOfflineNodes)
      }
    )

    entryNodes.start()

    await entryNodes.updatePublicNodes()

    await Promise.all(relayNodes.map((relayNode) => relayNode[0]))

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays != undefined, `must expose relay addresses`)
    assert(usedRelays.length == MAX_RELAYS_PER_NODE, `must expose ${MAX_RELAYS_PER_NODE} relay addresses`)

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == ENTRY_NODES_MAX_PARALLEL_DIALS + 1)
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

    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      network.connect as any,
      {}
    )

    entryNodes.start()

    entryNodes.uncheckedEntryNodes.push(newNode)

    let usedRelay = {
      relayDirectAddress: new Multiaddr('/ip4/127.0.0.1/tcp/1234'),
      ourCircuitAddress: new Multiaddr(`/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`)
    }

    entryNodes.usedRelays.push(usedRelay)

    // Should have one unchecked node and one relay node
    assert(entryNodes.getUsedRelayAddresses().length == 1)
    assert(entryNodes.getUncheckedEntryNodes().length == 1)

    const connectPromise = once(newNodeListener, 'connected')

    const updatePromise = once(entryNodes, RELAY_CHANGED_EVENT)

    entryNodes.onRemoveRelay(relay.id)

    await Promise.all([connectPromise, updatePromise])

    assert(entryNodes.getAvailabeEntryNodes().length == 1)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(entryNodes.getUsedRelayAddresses().length == 1)

    assert(
      usedRelays[0].equals(new Multiaddr(`/p2p/${newNode.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`))
    )

    newNodeListener.removeAllListeners()
    network.stop()
    entryNodes.stop()
  })

  it('take those nodes that are online', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)
    const relayListener = network.listen(relay.multiaddrs[0].toString())

    const connectPromise = once(relayListener, 'connected')

    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      network.connect as any,
      {}
    )

    entryNodes.start()

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
    assert(
      usedRelays[0].equals(new Multiaddr(`/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`))
    )

    network.stop()
    relayListener.removeAllListeners()
    entryNodes.stop()
  })

  it('no available entry nodes', async function () {
    const network = createFakeNetwork()

    const offlineRelay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      network.connect as any,
      {}
    )

    entryNodes.start()

    entryNodes.uncheckedEntryNodes.push(offlineRelay)

    await entryNodes.updatePublicNodes()

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 0)

    network.stop()
    entryNodes.stop()
  })

  it('do not emit listening event if nothing has changed', async function () {
    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      (async () => {}) as any,
      {}
    )

    entryNodes.start()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    let usedRelay = {
      relayDirectAddress: new Multiaddr(`/ip4/127.0.0.1/tcp/1`),
      ourCircuitAddress: new Multiaddr(`/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`)
    }

    entryNodes.availableEntryNodes.push({ ...relay, latency: 23 })
    entryNodes.usedRelays.push(usedRelay)

    entryNodes.once('listening', () =>
      assert.fail(`must not throw listening event if list of entry nodes has not changed`)
    )

    await entryNodes.updatePublicNodes()

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 0)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 0)

    entryNodes.stop()
  })

  it('events should trigger actions', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)
    const relayListener = network.listen(relay.multiaddrs[0].toString())

    const publicNodes = new EventEmitter() as PublicNodesEmitter
    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      network.connect as any,
      {
        publicNodes
      }
    )

    entryNodes.start()

    publicNodes.emit('addPublicNode', relay)

    await once(entryNodes, RELAY_CHANGED_EVENT)

    publicNodes.emit('removePublicNode', relay.id)

    await once(entryNodes, RELAY_CHANGED_EVENT)

    entryNodes.once(RELAY_CHANGED_EVENT, () => assert.fail('Must not throw the relay:changed event'))

    await entryNodes.updatePublicNodes()

    relayListener.removeAllListeners()
    entryNodes.stop()
    network.stop()
  })

  it('renew DHT entry', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    const connectEmitter = network.listen(relay.multiaddrs[0].toString())

    let renews = 0

    connectEmitter.on('connected', () => renews++)

    const publicNodes: PublicNodesEmitter = new EventEmitter()

    const CUSTOM_DHT_RENEWAL_TIMEOUT = 100 // very short timeout

    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      network.connect as any,
      {
        dhtRenewalTimeout: CUSTOM_DHT_RENEWAL_TIMEOUT,
        publicNodes
      }
    )

    entryNodes.start()

    publicNodes.emit('addPublicNode', relay)

    await new Promise((resolve) => setTimeout(resolve, 1e3))

    // depends on scheduler
    assert([9, 10].includes(renews), `Should capture at least 9 renews but not more than 10`)

    connectEmitter.removeAllListeners()
    entryNodes.stop()
    network.stop()
  })

  it('do not contact nodes we are already connected to', async function () {
    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      // Make sure that call is indeed asynchronous
      (async () => new Promise((resolve) => setImmediate(resolve))) as any,
      {}
    )

    const ma = new Multiaddr('/ip4/8.8.8.8/tcp/9091')

    const peerStoreEntry = getPeerStoreEntry(ma.toString())

    entryNodes.usedRelays.push({
      relayDirectAddress: ma,
      ourCircuitAddress: new Multiaddr(
        `/p2p/${peerStoreEntry.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`
      )
    })

    entryNodes.start()

    entryNodes.onNewRelay(peerStoreEntry)

    const uncheckedNodes = entryNodes.getUncheckedEntryNodes()

    assert(uncheckedNodes.length == 1, `Unchecked nodes must contain one entry`)
    assert(uncheckedNodes[0].id.equals(peerStoreEntry.id), `id must match the generated one`)
    assert(uncheckedNodes[0].multiaddrs.length == peerStoreEntry.multiaddrs.length, `must not contain more multiaddrs`)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 1, `must not expose any relay addrs`)

    entryNodes.stop()
  })

  it('reconnect on disconnect - temporarily offline', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/1.2.3.4/tcp/1`)
    const relayListener = network.listen(relay.multiaddrs[0].toString())

    let secondAttempt = defer<void>()
    let connectAttempt = 0
    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      (async (ma: Multiaddr, opts: any) => {
        switch (connectAttempt++) {
          case 0:
            return network.connect(ma, opts)
          case 1:
            return
          case 2:
            secondAttempt.resolve()
            return network.connect(ma, opts)
          default:
            return
        }
      }) as any,
      // Should be successful after second try
      {
        entryNodeReconnectBaseTimeout: 1,
        entryNodeReconnectBackoff: 5
      }
    )

    entryNodes.start()

    const updated = once(entryNodes, RELAY_CHANGED_EVENT)

    entryNodes.onNewRelay(relay)

    await updated

    // Should eventually remove relay from list
    network.close(relay.multiaddrs[0])

    await secondAttempt.promise

    // Wait for end of event loop
    await new Promise((resolve) => setImmediate(resolve))

    const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
    assert(availablePublicNodes.length == 1, `must keep entry node after reconnect`)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays.length == 1, `must keep relay address after reconnect`)

    relayListener.removeAllListeners()
    entryNodes.stop()
  })

  it('reconnect on disconnect - permanently offline', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/1.2.3.4/tcp/1`)
    const relayListener = network.listen(relay.multiaddrs[0].toString())

    let connectAttempt = 0

    const entryNodes = new TestingEntryNodes(
      peerId,
      {
        connectionManager: new FakeConnectionManager(true)
      },
      (async (ma: Multiaddr, opts: any) => {
        switch (connectAttempt++) {
          case 0:
            return network.connect(ma, opts)
          default:
            return
        }
      }) as any,
      // Should fail after second try
      {
        entryNodeReconnectBaseTimeout: 1,
        entryNodeReconnectBackoff: 5
      }
    )

    entryNodes.start()

    const firstUpdate = once(entryNodes, RELAY_CHANGED_EVENT)

    entryNodes.onNewRelay(relay)

    await firstUpdate

    const secondUpdate = once(entryNodes, RELAY_CHANGED_EVENT)

    // Should eventually remove relay from list
    network.close(relay.multiaddrs[0])

    await secondUpdate

    const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
    assert(availablePublicNodes.length == 0, `must remove node from public nodes`)

    const usedRelays = entryNodes.getUsedRelayAddresses()
    assert(usedRelays == undefined || usedRelays.length == 0, `must not expose any relay addrs`)

    relayListener.removeAllListeners()
    entryNodes.stop()
  })
})
