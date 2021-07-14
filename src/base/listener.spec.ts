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
import { once, EventEmitter } from 'events'

import { networkInterfaces } from 'os'
import { u8aEquals } from '@hoprnet/hopr-utils'

describe('check listening to sockets', function () {
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
    const socket = dgram.createSocket('udp4')

    await new Promise<void>((resolve, reject) => {
      socket.once('error', (err: any) => {
        socket.removeListener('listening', resolve)
        reject(err)
      })
      socket.once('listening', () => {
        socket.removeListener('error', reject)
        resolve()
      })

      try {
        socket.bind(port)
      } catch (err) {
        reject(err)
      }
    })

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

  async function startNode(
    publicNodes: EventEmitter = new EventEmitter(),
    initialNodes: Multiaddr[],
    state: { msgReceived: DeferredPromise<void>; expectedMessageReceived?: DeferredPromise<void> },
    expectedMessage?: Uint8Array,
    peerId?: PeerId
  ) {
    peerId = peerId ?? (await PeerId.create({ keyType: 'secp256k1' }))
    const listener = new Listener(
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
      publicNodes,
      initialNodes,
      peerId,
      undefined
    )

    await waitUntilListening(listener, new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))

    return {
      peerId,
      listener
    }
  }

  async function stopNode(socket: Socket | Listener) {
    const closePromise = once(socket, 'close')

    socket.close()

    return closePromise
  }

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
        stunServers.map((s: Socket) => new Multiaddr(`/ip4/127.0.0.1/tcp/${s.address().port}`)),
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

    const stunServerMultiaddr = new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)

    const publicNodesEmitter = new EventEmitter()
    const relay = await startNode(publicNodesEmitter, [stunServerMultiaddr], {
      msgReceived: relayContacted
    })

    const node = await startNode(publicNodesEmitter, [stunServerMultiaddr], { msgReceived: Defer() })

    publicNodesEmitter.emit(
      `publicNode`,
      new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)
    )

    // Checks that relay and STUN got contacted, otherwise timeout
    await relayContacted.promise

    // Let events happen
    await new Promise((resolve) => setTimeout(resolve))

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
      undefined,
      [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
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
      [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
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

    const node = await startNode(undefined, [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)], {
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

    const relay = await startNode(undefined, [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)], {
      msgReceived: Defer()
    })

    const publicNodesEmitter = new EventEmitter()

    const node = await startNode(
      publicNodesEmitter,
      [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      {
        msgReceived: Defer()
      }
    )

    publicNodesEmitter.emit(
      'publicNode',
      new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)
    )

    publicNodesEmitter.emit(
      'publicNode',
      new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)
    )

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))
    await new Promise((resolve) => setTimeout(resolve))

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
    const msgReceived = Defer<void>()
    const expectedMessageReceived = Defer<void>()

    const node = await startNode(undefined, [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)], {
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

    const relay = await startNode(undefined, [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)], {
      msgReceived: Defer()
    })

    const publicNodesEmitter = new EventEmitter()

    const node = await startNode(
      publicNodesEmitter,
      [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      {
        msgReceived: Defer()
      }
    )

    publicNodesEmitter.emit(
      `publicNode`,
      new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)
    )

    // Let events happen
    await new Promise((resolve) => setTimeout(resolve))

    let addrs = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(addrs.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`))

    publicNodesEmitter.emit(
      `publicNode`,
      new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId}`)
    )

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    let addrsAfterSecondEvent = node.listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(addrs.length == addrsAfterSecondEvent.length)

    assert(
      addrsAfterSecondEvent.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`)
    )

    // Stop first relay and let it attach to different port
    await stopNode(relay.listener)

    const newRelay = await startNode(
      undefined,
      [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      {
        msgReceived: Defer()
      },
      undefined,
      relay.peerId
    )

    publicNodesEmitter.emit(
      `publicNode`,
      new Multiaddr(`/ip4/127.0.0.1/tcp/${newRelay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)
    )

    // Let Events happen
    await new Promise((resolve) => setTimeout(resolve))

    const addrsAfterThirdEvent = node.listener.getAddrs()

    assert(addrsAfterSecondEvent.length == addrsAfterThirdEvent.length)

    await Promise.all([stopNode(node.listener), stopNode(newRelay.listener), stopNode(stunServer)])
  })
})
