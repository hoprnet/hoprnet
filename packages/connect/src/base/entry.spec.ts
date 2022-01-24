import { EntryNodes, ENTRY_NODES_MAX_PARALLEL_DIALS, RELAY_CHANGED_EVENT } from './entry'
import type PeerId from 'peer-id'
import assert from 'assert'
import { createPeerId, getPeerStoreEntry } from './utils.spec'
import { once, EventEmitter } from 'events'
import { Multiaddr } from 'multiaddr'

import { MAX_RELAYS_PER_NODE, OK } from '../constants'
import type { PeerStoreType, PublicNodesEmitter } from '../types'

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
function createFakeNetwork() {
  const network = new EventEmitter()

  const listen = (addr: string) => {
    const emitter = new EventEmitter()
    network.on(connectEvent(addr), () => emitter.emit('connected'))

    return emitter
  }

  const connect = (addr: string) => {
    if (network.listeners(connectEvent(addr)).length >= 1) {
      network.emit(connectEvent(addr))
      return Promise.resolve({
        newStream: (_protocols: string[]) =>
          Promise.resolve({
            stream: {
              source: (async function* () {
                yield OK
              })()
            }
          })
      })
    } else {
      return Promise.resolve(undefined)
    }
  }

  return {
    listen,
    connect,
    close: network.removeAllListeners.bind(network)
  }
}

describe('entry node functionality', function () {
  const peerId = createPeerId()
  it('add public nodes', function () {
    const entryNodes = new TestingEntryNodes(
      peerId,
      // Make sure that call is indeed asynchronous
      (async () => new Promise((resolve) => setImmediate(resolve))) as any,
      {}
    )

    const peerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/0`)

    entryNodes.onNewRelay(peerStoreEntry)
    // Should filter duplicate
    entryNodes.onNewRelay(peerStoreEntry)

    const uncheckedNodes = entryNodes.getUncheckedEntryNodes()

    assert(uncheckedNodes.length == 1, `Unchecked nodes must contain one entry`)
    assert(uncheckedNodes[0].id.equals(peerStoreEntry.id), `id must match the generated one`)
    assert(uncheckedNodes[0].multiaddrs.length == peerStoreEntry.multiaddrs.length, `must not contain more multiaddrs`)

    const usedRelays = entryNodes.getUsedRelays()
    assert(usedRelays == undefined || usedRelays.length == 0, `must not expose any internal addrs`)

    entryNodes.stop()
  })

  it('remove an offline node', function () {
    const entryNodes = new TestingEntryNodes(
      peerId,
      (async () => new Promise((resolve) => setImmediate(resolve))) as any,
      {}
    )

    const peerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/0`)

    entryNodes.availableEntryNodes.push({
      ...peerStoreEntry,
      latency: 23
    })

    entryNodes.onRemoveRelay(peerStoreEntry.id)

    const availablePublicNodes = entryNodes.getAvailabeEntryNodes()
    assert(availablePublicNodes.length == 0, `must remove node from public nodes`)

    const usedRelays = entryNodes.getUsedRelays()
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
      async (ma: Multiaddr) => (await network.connect(ma.toString())) as any,
      {
        initialNodes: [relay]
      }
    )

    await entryNodes.updatePublicNodes()

    await connectPromise

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 1, `must contain exactly one public node`)
    assert(availableEntryNodes[0].id.equals(relay.id), `must contain correct peerId`)
    assert(availableEntryNodes[0].latency >= 0, `latency must be non-negative`)

    const usedRelays = entryNodes.getUsedRelays()
    assert(usedRelays != undefined, `must expose relay addrs`)
    assert(usedRelays.length == 1, `must expose exactly one relay addrs`)
    assert(
      usedRelays[0].toString() === `/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`,
      `must expose the right relay address`
    )

    relayListener.removeAllListeners()
    network.close()
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
      async (ma: Multiaddr) => (await network.connect(ma.toString())) as any,
      {
        initialNodes: relayNodes.map((relayNode) => relayNode[1]).concat(additionalOfflineNodes)
      }
    )

    await entryNodes.updatePublicNodes()

    await Promise.all(relayNodes.map((relayNode) => relayNode[0]))

    const usedRelays = entryNodes.getUsedRelays()
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
    network.close()
    entryNodes.stop()
  })

  it('update nodes once node became offline', async function () {
    const network = createFakeNetwork()

    const newNode = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)
    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`)

    const newNodeListener = network.listen(newNode.multiaddrs[0].toString())

    const entryNodes = new TestingEntryNodes(
      peerId,
      async (ma: Multiaddr) => (await network.connect(ma.toString())) as any,
      {}
    )

    entryNodes.uncheckedEntryNodes.push(newNode)

    entryNodes.usedRelays.push(new Multiaddr(`/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`))

    // Should have one unchecked node and one relay node
    assert(entryNodes.getUsedRelays().length == 1)
    assert(entryNodes.getUncheckedEntryNodes().length == 1)

    const connectPromise = once(newNodeListener, 'connected')

    const updatePromise = once(entryNodes, RELAY_CHANGED_EVENT)
    entryNodes.onRemoveRelay(relay.id)

    await Promise.all([connectPromise, updatePromise])

    assert(entryNodes.getAvailabeEntryNodes().length == 1)

    const usedRelays = entryNodes.getUsedRelays()
    assert(entryNodes.getUsedRelays().length == 1)

    assert(
      usedRelays[0].equals(new Multiaddr(`/p2p/${newNode.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`))
    )

    newNodeListener.removeAllListeners()
    network.close()
    entryNodes.stop()
  })

  it('take those nodes that are online', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)
    const relayListener = network.listen(relay.multiaddrs[0].toString())

    const connectPromise = once(relayListener, 'connected')

    const entryNodes = new TestingEntryNodes(
      peerId,
      async (ma: Multiaddr) => (await network.connect(ma.toString())) as any,
      {}
    )

    const fakeNode = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/2`)

    entryNodes.uncheckedEntryNodes.push(relay)
    entryNodes.uncheckedEntryNodes.push(fakeNode)

    await entryNodes.updatePublicNodes()

    await connectPromise

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 1)
    assert(availableEntryNodes[0].id.equals(relay.id))

    const usedRelays = entryNodes.getUsedRelays()
    assert(entryNodes.getUsedRelays().length == 1)
    assert(
      usedRelays[0].equals(new Multiaddr(`/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`))
    )

    network.close()
    relayListener.removeAllListeners()
    entryNodes.stop()
  })

  it('do not emit listening event if nothing has changed', async function () {
    const entryNodes = new TestingEntryNodes(peerId, (async () => {}) as any, {})

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    entryNodes.availableEntryNodes.push({ ...relay, latency: 23 })
    entryNodes.usedRelays.push(new Multiaddr(`/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`))

    entryNodes.once('listening', () =>
      assert.fail(`must not throw listening event if list of entry nodes has not changed`)
    )

    await entryNodes.updatePublicNodes()

    const availableEntryNodes = entryNodes.getAvailabeEntryNodes()
    assert(availableEntryNodes.length == 1)
    assert(availableEntryNodes[0].id.equals(relay.id))

    const usedRelays = entryNodes.getUsedRelays()
    assert(usedRelays.length == 1)
    assert(
      usedRelays[0].equals(new Multiaddr(`/p2p/${relay.id.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`))
    )
    entryNodes.stop()
  })

  it('events should trigger actions', async function () {
    const network = createFakeNetwork()

    const relay = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/1`)

    const publicNodes = new EventEmitter() as PublicNodesEmitter
    const entryNodes = new TestingEntryNodes(
      peerId,
      async (ma: Multiaddr) => (await network.connect(ma.toString())) as any,
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

    entryNodes.stop()
    network.close()
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
      async (ma: Multiaddr) => (await network.connect(ma.toString())) as any,
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
    network.close()
  })
})
