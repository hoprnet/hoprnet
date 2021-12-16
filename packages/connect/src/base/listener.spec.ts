import assert from 'assert'
import { Listener } from './listener'
import { Multiaddr } from 'multiaddr'
import type { MultiaddrConnection, Upgrader } from 'libp2p-interfaces/transport'
import dgram, { type Socket } from 'dgram'
import PeerId from 'peer-id'
import { createConnection } from 'net'
import * as stun from 'webrtc-stun'
import { once, on, EventEmitter } from 'events'

import { type NetworkInterfaceInfo, networkInterfaces } from 'os'
import { u8aEquals, defer, type DeferType, toNetworkPrefix, u8aAddrToString } from '@hoprnet/hopr-utils'

import type { PublicNodesEmitter, PeerStoreType } from '../types'

import { waitUntilListening, stopNode, startStunServer } from './utils.spec'

/**
 * Decorated Listener class that emits events after
 * updating list of potential relays
 */
class TestingListener extends Listener {
  public emitter: EventEmitter
  constructor(...args: ConstructorParameters<typeof Listener>) {
    super(...args)

    this.emitter = new EventEmitter()
  }

  protected onRemoveRelay(peer: PeerId) {
    super.onRemoveRelay(peer)

    this.emitter.emit(`_nodeOffline`, peer)
  }

  protected async updatePublicNodes(peer: PeerId) {
    await super.updatePublicNodes(peer)

    this.emitter.emit(`_newNodeRegistered`, peer)
  }
}

async function getPeerStoreEntry(addr: string): Promise<PeerStoreType> {
  return {
    id: await PeerId.create({ keyType: 'secp256k1' }),
    multiaddrs: [new Multiaddr(addr)]
  }
}

/**
 * Creates a node and attaches message listener to it.
 * @param publicNodes emitter that emit an event on new public nodes
 * @param initialNodes nodes that are initially known
 * @param state check message reception and content of message
 * @param expectedMessage message to check for, or undefined to skip this check
 * @param peerId peerId of the node
 * @returns
 */
async function startNode(
  initialNodes: PeerStoreType[],
  state?: { msgReceived?: DeferType<void>; expectedMessageReceived?: DeferType<void> },
  expectedMessage?: Uint8Array,
  peerId?: PeerId,
  upgrader?: Upgrader,
  handler?: (conn: any) => any | Promise<any>,
  runningLocally?: boolean
) {
  peerId = peerId ?? (await PeerId.create({ keyType: 'secp256k1' }))
  const publicNodesEmitter = new EventEmitter() as PublicNodesEmitter

  const listener = new TestingListener(
    handler,
    upgrader ??
      ({
        upgradeInbound: async (conn: MultiaddrConnection) => {
          if (expectedMessage != undefined) {
            for await (const msg of conn.source) {
              if (u8aEquals(msg.slice(), expectedMessage)) {
                state?.expectedMessageReceived?.resolve()
              }
            }
          }

          state?.msgReceived?.resolve()
          return conn
        },
        upgradeOutbound: async (conn: MultiaddrConnection) => conn
      } as any),
    publicNodesEmitter,
    initialNodes,
    peerId,
    undefined,
    runningLocally ?? false
  )

  process.nextTick(() =>
    waitUntilListening(listener, new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId!.toB58String()}`))
  )

  const initialNodesRegistered: PeerId[] = []

  for await (const initialNode of on(listener.emitter, '_newNodeRegistered')) {
    if (initialNodesRegistered.push(initialNode[0]) == initialNodes.length) {
      break
    }
  }

  assert(
    initialNodes.every((entry: PeerStoreType) => initialNodesRegistered.some((peer: PeerId) => peer.equals(entry.id)))
  )

  return {
    peerId,
    listener,
    publicNodesEmitter
  }
}

describe('check listening to sockets', function () {
  it('recreate the socket and perform STUN requests', async function () {
    this.timeout(10e3) // 3 seconds should be more than enough

    let listener: Listener
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const AMOUNT = 3

    const msgReceived = Array.from({ length: AMOUNT }, (_) => defer<void>())

    const stunServers = await Promise.all(
      Array.from({ length: AMOUNT }, (_, index: number) =>
        startStunServer(undefined, { msgReceived: msgReceived[index] })
      )
    )

    const peerStoreEntries = await Promise.all(
      stunServers.map((s: Socket) => getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${s.address().port}`))
    )

    let port: number | undefined

    for (let i = 0; i < 3; i++) {
      listener = new Listener(undefined, undefined as any, undefined, [peerStoreEntries[i]], peerId, undefined, false)

      let listeningMultiaddr: Multiaddr
      if (port != undefined) {
        listeningMultiaddr = new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`)
      } else {
        // Listen to previously used port
        listeningMultiaddr = new Multiaddr(`/ip4/127.0.0.1/tcp/${port}/p2p/${peerId.toB58String()}`)
      }

      await waitUntilListening(listener, listeningMultiaddr)
      if (port == undefined) {
        // Store the port to which we have listened before
        port = listener.getPort()
      }
      assert(port != undefined)
      await stopNode(listener)
    }

    await Promise.all(msgReceived.map((received) => received.promise))
    await Promise.all(stunServers.map(stopNode))
  })

  it('should contact potential relays and expose relay addresses', async function () {
    this.timeout(4e3)

    const relayContacted = defer<void>()

    const stunServer = await startStunServer(undefined)

    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer], {
      msgReceived: relayContacted
    })

    const node = await startNode([stunPeer], { msgReceived: defer<void>() })

    const eventPromise = once(node.listener.emitter, '_newNodeRegistered')

    node.publicNodesEmitter.emit(`addPublicNode`, {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    // Checks that relay and STUN got contacted, otherwise timeout
    await Promise.all([relayContacted.promise, eventPromise])

    const addrs = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(
      addrs.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`),
      `Listener must expose circuit address`
    )

    await Promise.all([stopNode(node.listener), stopNode(relay.listener), stopNode(stunServer)])
  })

  it('check that node is reachable', async function () {
    const stunServer = await startStunServer(undefined)
    const msgReceived = defer<void>()
    const expectedMessageReceived = defer<void>()

    const testMessage = new TextEncoder().encode('test')

    const node = await startNode(
      [await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      {
        msgReceived,
        expectedMessageReceived
      },
      testMessage
    )

    const socket = createConnection(
      {
        host: '127.0.0.1',
        port: node.listener.getPort()
      },
      () => {
        socket.write(testMessage, () => {
          socket.end()
        })
      }
    )

    await msgReceived.promise

    await Promise.all([stopNode(node.listener), stopNode(stunServer)])
  })

  it('should bind to specific interfaces', async function () {
    // Test does do not do anything if there are only IPv6 addresses
    const usableInterfaces = networkInterfaces()

    for (const iface of Object.keys(usableInterfaces)) {
      const osIface = usableInterfaces[iface]

      if (osIface == undefined || osIface.some((x) => x.internal) || !osIface.some((x) => x.family == 'IPv4')) {
        delete usableInterfaces[iface]
      }
    }

    if (Object.keys(usableInterfaces).length == 0) {
      // Cannot test without any available interfaces
      return
    }

    const firstUsableInterfaceName = Object.keys(usableInterfaces)[0]

    const address = (usableInterfaces[firstUsableInterfaceName] as NetworkInterfaceInfo[]).filter((addr) => {
      if (addr.internal) {
        return false
      }

      if (addr.family == 'IPv6') {
        return false
      }

      return true
    })[0]

    const network = toNetworkPrefix(address)

    const notUsableAddress = network.networkPrefix.slice()
    // flip first bit of the address
    notUsableAddress[0] ^= 128

    const stunServer = await startStunServer(undefined)
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const listener = new Listener(
      undefined,
      {
        upgradeInbound: async (conn: MultiaddrConnection) => conn
      } as any,
      undefined,
      [await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      peerId,
      firstUsableInterfaceName,
      false
    )

    await assert.rejects(
      () =>
        listener.listen(
          new Multiaddr(`/ip4/${u8aAddrToString(notUsableAddress, address.family)}/tcp/0/p2p/${peerId.toB58String()}`)
        ),
      `Must throw if we can't bind to an unusable address`
    )

    await assert.doesNotReject(
      async () => await listener.listen(new Multiaddr(`/ip4/${address.address}/tcp/0/p2p/${peerId.toB58String()}`)),
      `Must be able to bind to correct address`
    )

    await Promise.all([stopNode(listener), stopNode(stunServer)])
  })

  it('check that node speaks STUN', async function () {
    const msgReceived = defer<void>()
    const stunServer = await startStunServer(undefined)

    const node = await startNode([await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)])

    const socket = dgram.createSocket({ type: 'udp4' })
    const tid = stun.generateTransactionId()

    socket.on('message', (msg) => {
      const res = stun.createBlank()

      // if msg is valid STUN message
      if (res.loadBuffer(msg)) {
        // if msg is BINDING_RESPONSE_SUCCESS and valid content
        if (res.isBindingResponseSuccess({ transactionId: tid })) {
          const attr = res.getXorMappedAddressAttribute()
          // if msg includes attr
          if (attr) {
            msgReceived.resolve()
          }
        }
      }

      socket.close()
    })

    const req = stun.createBindingRequest(tid).setFingerprintAttribute()

    const addrs = node.listener.getAddrs()

    const localAddress = addrs.find((ma: Multiaddr) => ma.toString().match(/127.0.0.1/))

    assert(localAddress != null, `Listener must be available on localhost`)

    socket.send(req.toBuffer(), localAddress.toOptions().port, `localhost`)

    await msgReceived.promise

    await stopNode(node.listener)
    await stopNode(stunServer)
  })

  it('get the right addresses', async function () {
    this.timeout(4e3) // for CI
    const stunServer = await startStunServer(undefined)

    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)
    const relay = await startNode([stunPeer])

    const node = await startNode([stunPeer])

    let eventPromise = once(node.listener.emitter, '_newNodeRegistered')
    node.publicNodesEmitter.emit('addPublicNode', {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    await eventPromise

    eventPromise = once(node.listener.emitter, '_newNodeRegistered')
    node.publicNodesEmitter.emit('addPublicNode', {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    await eventPromise

    const addrsFromListener = node.listener.getAddrs()

    const uniqueAddresses = new Set<string>(addrsFromListener.map((ma: Multiaddr) => ma.toString()))

    assert(
      uniqueAddresses.has(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`),
      `Addresses must include relay address`
    )

    assert(
      uniqueAddresses.has(`/ip4/127.0.0.1/tcp/${node.listener.getPort()}/p2p/${node.peerId.toB58String()}`),
      `Addresses must include relay address`
    )

    assert(addrsFromListener.length == uniqueAddresses.size, `Addresses must not appear twice`)

    await Promise.all([stopNode(relay.listener), stopNode(node.listener), stopNode(stunServer)])
  })

  it('check connection tracking', async function () {
    const stunServer = await startStunServer(undefined)
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)
    const msgReceived = defer<void>()
    const expectedMessageReceived = defer<void>()

    const node = await startNode([stunPeer], {
      msgReceived,
      expectedMessageReceived
    })

    const bothConnectionsOpened = defer()
    let connections = 0

    node.listener.on('connection', () => {
      connections++

      if (connections == 2) {
        bothConnectionsOpened.resolve()
      }
    })

    const socketOne = createConnection({
      host: '127.0.0.1',
      port: node.listener.getPort()
    })

    const socketTwo = createConnection({
      host: '127.0.0.1',
      port: node.listener.getPort()
    })

    await bothConnectionsOpened.promise

    assert(node.listener.getConnections() == 2)

    // Add event listener at the end of the event listeners array
    const socketOneClosePromise = once(socketOne, 'close')
    const socketTwoClosePromise = once(socketTwo, 'close')

    socketOne.end()
    socketTwo.end()

    await Promise.all([socketOneClosePromise, socketTwoClosePromise])

    // let I/O actions happen
    await new Promise((resolve) => setImmediate(resolve))

    assert(node.listener.getConnections() == 0, `Connection must have been removed`)

    await Promise.all([stopNode(node.listener), stopNode(stunServer)])
  })

  it('add relay node only once', async function () {
    const stunServer = await startStunServer(undefined)
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer])

    const node = await startNode([stunPeer])

    let eventPromise = once(node.listener.emitter, '_newNodeRegistered')

    node.publicNodesEmitter.emit(`addPublicNode`, {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    await eventPromise

    let addrs = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(
      addrs.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`),
      'Addrs should include new relay node'
    )

    eventPromise = once(node.listener.emitter, '_newNodeRegistered')
    node.publicNodesEmitter.emit(`addPublicNode`, {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    await eventPromise

    let addrsAfterSecondEvent = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(addrs.length == addrsAfterSecondEvent.length)

    assert(
      addrsAfterSecondEvent.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`),
      'Addrs should include new relay node'
    )

    await Promise.all([stopNode(node.listener), stopNode(relay.listener), stopNode(stunServer)])
  })

  it('overwrite existing relays', async function () {
    this.timeout(4e3) // for CI
    const stunServer = await startStunServer(undefined)
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer])

    const node = await startNode([stunPeer])

    let eventPromise = once(node.listener.emitter, '_newNodeRegistered')

    node.publicNodesEmitter.emit(`addPublicNode`, {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    await eventPromise

    eventPromise = once(node.listener.emitter, '_newNodeRegistered')

    let addrs = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(addrs.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`))

    node.publicNodesEmitter.emit(`addPublicNode`, {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    await eventPromise

    // Stop first relay and let it attach to different port
    await stopNode(relay.listener)

    const newRelay = await startNode([stunPeer], undefined, undefined, relay.peerId)

    eventPromise = once(node.listener.emitter, '_newNodeRegistered')
    node.publicNodesEmitter.emit(`addPublicNode`, {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${newRelay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    await eventPromise

    const addrsAfterThirdEvent = node.listener.getAddrs()

    assert(addrs.length == addrsAfterThirdEvent.length)

    await Promise.all([stopNode(node.listener), stopNode(newRelay.listener), stopNode(stunServer)])
  })

  it('remove offline relay nodes', async function () {
    this.timeout(4e3) // for CI
    const stunServer = await startStunServer(undefined)
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer])

    const node = await startNode([stunPeer])

    let eventPromise = once(node.listener.emitter, '_newNodeRegistered')

    node.publicNodesEmitter.emit(`addPublicNode`, {
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)]
    })

    await eventPromise

    let addrs = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(addrs.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`))

    eventPromise = once(node.listener.emitter, '_nodeOffline')

    node.publicNodesEmitter.emit(`removePublicNode`, relay.peerId)

    await eventPromise

    let addrsAfterRemoval = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(addrs.length - 1 == addrsAfterRemoval.length, 'Addr should be removed, hence size should be reduced by one.')
    assert(
      !addrsAfterRemoval.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`),
      'Addrs should not contain removed node'
    )

    await Promise.all([stopNode(node.listener), stopNode(relay.listener), stopNode(stunServer)])
  })

  it('remove offline relay nodes - edge cases', async function () {
    this.timeout(4e3) // for CI
    const stunServer = await startStunServer(undefined)
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer])

    const node = await startNode([stunPeer])

    let addrs = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    let eventPromise = once(node.listener.emitter, '_nodeOffline')

    node.publicNodesEmitter.emit(`removePublicNode`, relay.peerId)

    await eventPromise

    let addrsAfterRemoval = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(
      addrs.length == addrsAfterRemoval.length,
      'Number of addresses should stay same after removing invalid multiaddr'
    )
    assert(
      !addrsAfterRemoval.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`),
      'Addrs should not include addr of invalid node'
    )
    assert(
      addrs.every((addr: string) => addrsAfterRemoval.some((addrAfterRemoval: string) => addr === addrAfterRemoval)),
      'Addrs should stay same after trying remove invalid multiaddr'
    )

    await Promise.all([stopNode(node.listener), stopNode(relay.listener), stopNode(stunServer)])
  })
})

describe('error cases', function () {
  it('throw error while upgrading the connection', async () => {
    const peer = await PeerId.create({ keyType: 'secp256k1' })
    const stunServer = await startStunServer(undefined)

    const node = await startNode(
      [await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      undefined,
      undefined,
      peer,
      {
        upgradeInbound: async (_maConn: MultiaddrConnection) => {
          await new Promise((resolve) => setTimeout(resolve, 100))

          throw Error('foo')
        }
      } as any
    )

    const socket = createConnection(
      {
        host: '127.0.0.1',
        port: node.listener.getPort()
      },
      async () => {
        await new Promise((resolve) => setTimeout(resolve, 200))

        socket.end()
      }
    )

    await new Promise((resolve) => setTimeout(resolve, 300))

    await Promise.all([node.listener, stunServer].map(stopNode))
  })

  it('throw unexpected error', async function () {
    // This unit test case produces an uncaught error in case there
    // is no "global" try / catch on incoming socket connections
    const peer = await PeerId.create({ keyType: 'secp256k1' })
    const stunServer = await startStunServer(undefined)

    const node = await startNode(
      [await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      undefined,
      undefined,
      peer,
      {
        upgradeInbound: async (_maConn: MultiaddrConnection) => {
          await new Promise((resolve) => setTimeout(resolve, 100))

          return {}
        }
      } as any,
      // Simulate an unexpected error while processing data
      (conn: any) => conn.nonExisting()
    )

    const socket = createConnection(
      {
        host: '127.0.0.1',
        port: node.listener.getPort()
      },
      async () => {
        await new Promise((resolve) => setTimeout(resolve, 200))

        socket.end()
      }
    )

    await new Promise((resolve) => setTimeout(resolve, 300))

    await Promise.all([node.listener, stunServer].map(stopNode))
  })
})
