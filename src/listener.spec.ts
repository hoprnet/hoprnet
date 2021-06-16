/// <reference path="./@types/libp2p.ts" />

import assert from 'assert'
import Listener from './listener'
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
import { once } from 'events'

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
      socket.once('listening', resolve)

      try {
        socket.bind(port, () => {
          socket.removeListener('error', reject)
          resolve()
        })
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

  async function stopStunServer(socket: Socket) {
    await new Promise<void>((resolve) => {
      socket.once('close', resolve)
      socket.close()
    })
  }

  async function waitUntilListening(socket: Listener, ma: Multiaddr) {
    const promise = once(socket, 'listening')

    await socket.listen(ma)

    return promise
  }

  async function startNode(
    stunServers: Multiaddr[],
    state: { msgReceived: DeferredPromise<void>; expectedMessageReceived?: DeferredPromise<void> },
    expectedMessage?: Uint8Array
  ) {
    const peerId = await PeerId.create({ keyType: 'secp256k1' })
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
      stunServers,
      undefined,
      peerId,
      undefined
    )

    await waitUntilListening(listener, new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))

    return {
      peerId,
      listener
    }
  }

  async function stopListener(socket: Listener) {
    const promise = once(socket, 'close')

    await socket.close()

    return promise
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
        [
          new Multiaddr(`/ip4/127.0.0.1/udp/${stunServers[0].address().port}`),
          new Multiaddr(`/ip4/127.0.0.1/udp/${stunServers[1].address().port}`)
        ],
        undefined,
        peerId,
        undefined
      )

      await waitUntilListening(listener, new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))
      await stopListener(listener)
    }

    await Promise.all(msgReceived.map((received) => received.promise))

    await Promise.all(stunServers.map((s) => stopStunServer(s)))
  })

  it('should contact potential relays and expose relay addresses', async function () {
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const relayContacted = Defer<void>()

    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })

    const relay = await startNode([new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)], {
      msgReceived: relayContacted
    })

    const listener = new Listener(
      undefined,
      {
        upgradeOutbound: async (maConn: MultiaddrConnection) => maConn
      } as unknown as Upgrader,
      [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)],
      peerId,
      undefined
    )

    await waitUntilListening(listener, new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))

    // Checks that relay and STUN got contacted, otherwise timeout
    await relayContacted.promise

    const addrs = listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(
      addrs.includes(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`),
      `Listener must expose circuit address`
    )

    await Promise.all([
      // prettier-ignore
      stopListener(listener),
      stopListener(relay.listener),
      stopStunServer(stunServer)
    ])
  })

  it('check that node is reachable', async function () {
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })
    const msgReceived = Defer<void>()
    const expectedMessageReceived = Defer<void>()

    const testMessage = new TextEncoder().encode('test')

    const node = await startNode(
      [new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      {
        msgReceived,
        expectedMessageReceived
      },
      testMessage
    )

    new Promise<void>((resolve) => {
      const socket = net.createConnection(
        {
          host: '127.0.0.1',
          port: node.listener.getPort()
        },
        () => {
          socket.write(testMessage, () => {
            socket.end()
            resolve()
          })
        }
      )
    })

    await msgReceived.promise

    await Promise.all([stopListener(node.listener), stopStunServer(stunServer)])
  })

  it('should bind to specific interfaces', async function () {
    const validInterfaces = Object.keys(networkInterfaces()).filter((iface) =>
      networkInterfaces()[iface]?.some((x) => !x.internal)
    )

    if (validInterfaces.length == 0) {
      // Cannot test without any available interfaces
      return
    }

    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const listener = new Listener(
      undefined,
      {
        upgradeInbound: async (conn: MultiaddrConnection) => conn
      } as unknown as Upgrader,
      undefined,
      undefined,
      peerId,
      validInterfaces[0]
    )

    await assert.rejects(async () => {
      await waitUntilListening(listener, new Multiaddr(`/ip4/0.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))
    })

    await stopListener(listener)
  })

  it('check that node speaks STUN', async function () {
    const defer = Defer<void>()
    const stunServer = await startStunServer(undefined, { msgReceived: Defer() })

    const node = await startNode([new Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)], {
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

    await stopListener(node.listener)
  })

  // @TODO add test for connection tracking
})
