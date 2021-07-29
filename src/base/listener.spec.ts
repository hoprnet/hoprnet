/// <reference path="../@types/libp2p.ts" />

import assert from 'assert'
import { Listener } from './listener'
import { Multiaddr } from 'multiaddr'
import type { MultiaddrConnection, Upgrader } from 'libp2p'
import dgram from 'dgram'
import type { Socket, RemoteInfo } from 'dgram'
import { handleStunRequest } from './stun'
import PeerId from 'peer-id'
import net from 'net'
import Defer from 'p-defer'
import type { DeferredPromise } from 'p-defer'
import * as stun from 'webrtc-stun'
import { once, on, EventEmitter } from 'events'

import { networkInterfaces } from 'os'
import { u8aEquals } from '@hoprnet/hopr-utils'

import type { PublicNodesEmitter, PeerStoreType } from '../types'

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
 * Creates a UDP socket and binds it to the given port.
 * @param port port to which the socket should be bound
 * @returns a bound socket
 */
function bindToUdpSocket(port?: number): Promise<Socket> {
  const socket = dgram.createSocket('udp4')

  return new Promise<Socket>((resolve, reject) => {
    socket.once('error', (err: any) => {
      socket.removeListener('listening', resolve)
      reject(err)
    })
    socket.once('listening', () => {
      socket.removeListener('error', reject)
      resolve(socket)
    })

    try {
      socket.bind(port)
    } catch (err) {
      reject(err)
    }
  })
}

/**
 * Encapsulates the logic that is necessary to lauch a test
 * STUN server instance and track whether it receives requests
 * @param port port to listen to
 * @param state used to track incoming messages
 */
async function startStunServer(
  port: number | undefined,
  state: { msgReceived: DeferredPromise<void> }
): Promise<Socket> {
  const socket = await bindToUdpSocket(port)

  socket.on('message', (msg: Buffer, rinfo: RemoteInfo) => {
    state.msgReceived.resolve()
    handleStunRequest(socket, msg, rinfo)
  })

  return socket
}

async function waitUntilListening(socket: Listener, ma: Multiaddr) {
  const promise = once(socket, 'listening')

  await socket.listen(ma)

  return promise
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
  state: { msgReceived: DeferredPromise<void>; expectedMessageReceived?: DeferredPromise<void> },
  expectedMessage?: Uint8Array,
  peerId?: PeerId
) {
  peerId = peerId ?? (await PeerId.create({ keyType: 'secp256k1' }))
  const publicNodesEmitter = new EventEmitter() as PublicNodesEmitter

  const listener = new TestingListener(
    undefined,
    {
      upgradeInbound: async (conn: MultiaddrConnection) => {
        if (expectedMessage != undefined) {
          for await (const msg of conn.source) {
            if (u8aEquals(msg.slice(), expectedMessage)) {
              state.expectedMessageReceived?.resolve()
            }
          }
        }

        state.msgReceived.resolve()
        return conn
      },
      upgradeOutbound: async (conn: MultiaddrConnection) => conn
    } as unknown as Upgrader,
    publicNodesEmitter,
    initialNodes,
    peerId,
    undefined
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

async function stopNode(socket: Socket | Listener) {
  const closePromise = once(socket, 'close')

  socket.close()

  return closePromise
}

describe('check listening to sockets', function () {
  it('recreate the socket and perform STUN request', async function () {
    let listener: Listener
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const msgReceived = [Defer<void>(), Defer<void>()]

    const stunServers = [
      await startStunServer(undefined, { msgReceived: msgReceived[0] }),
      await startStunServer(undefined, { msgReceived: msgReceived[1] })
    ]

    for (let i = 0; i < 2; i++) {
      listener = new Listener(
        () => {},
        undefined as unknown as Upgrader,
        undefined,
        await Promise.all(stunServers.map((s: Socket) => getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${s.address().port}`))),
        peerId,
        undefined
      )

      await waitUntilListening(listener, new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))
      await stopNode(listener)
    }

    await Promise.all(msgReceived.map((received) => received.promise))

    await Promise.all(stunServers.map(stopNode))
  })

  it('should contact potential relays and expose relay addresses', async function () {
    const relayContacted = Defer<void>()

    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })

    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer], {
      msgReceived: relayContacted
    })

    const node = await startNode([stunPeer], { msgReceived: Defer() })

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
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })
    const msgReceived = Defer<void>()
    const expectedMessageReceived = Defer<void>()

    const testMessage = new TextEncoder().encode('test')

    const node = await startNode(
      [await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      {
        msgReceived,
        expectedMessageReceived
      },
      testMessage
    )

    const socket = net.createConnection(
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
    const validInterfaces = Object.keys(networkInterfaces()).filter((iface) =>
      networkInterfaces()[iface]?.some((x) => !x.internal)
    )

    if (validInterfaces.length == 0) {
      // Cannot test without any available interfaces
      return
    }

    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const listener = new Listener(
      undefined,
      {
        upgradeInbound: async (conn: MultiaddrConnection) => conn
      } as unknown as Upgrader,
      undefined,
      [await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      peerId,
      validInterfaces[0]
    )

    await assert.rejects(async () => {
      await waitUntilListening(listener, new Multiaddr(`/ip4/0.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))
    })

    await Promise.all([stopNode(listener), stopNode(stunServer)])
  })

  it('check that node speaks STUN', async function () {
    const defer = Defer<void>()
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })

    const node = await startNode([await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)], {
      msgReceived: Defer()
    })

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
            defer.resolve()
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

    await defer.promise

    await stopNode(node.listener)
  })

  it('get the right addresses', async function () {
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })

    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)
    const relay = await startNode([stunPeer], {
      msgReceived: Defer()
    })

    const node = await startNode([stunPeer], {
      msgReceived: Defer()
    })

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
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)
    const msgReceived = Defer<void>()
    const expectedMessageReceived = Defer<void>()

    const node = await startNode([stunPeer], {
      msgReceived,
      expectedMessageReceived
    })

    const bothConnectionsOpened = Defer()
    let connections = 0

    node.listener.on('connection', () => {
      connections++

      if (connections == 2) {
        bothConnectionsOpened.resolve()
      }
    })

    const socketOne = net.createConnection({
      host: '127.0.0.1',
      port: node.listener.getPort()
    })

    const socketTwo = net.createConnection({
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
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer], {
      msgReceived: Defer()
    })

    const node = await startNode([stunPeer], {
      msgReceived: Defer()
    })

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
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer], {
      msgReceived: Defer()
    })

    const node = await startNode([stunPeer], {
      msgReceived: Defer()
    })

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

    const newRelay = await startNode(
      [stunPeer],
      {
        msgReceived: Defer()
      },
      undefined,
      relay.peerId
    )

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
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer], {
      msgReceived: Defer()
    })

    const node = await startNode([stunPeer], {
      msgReceived: Defer()
    })

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
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })
    const stunPeer = await getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const relay = await startNode([stunPeer], {
      msgReceived: Defer()
    })

    const node = await startNode([stunPeer], {
      msgReceived: Defer()
    })

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
