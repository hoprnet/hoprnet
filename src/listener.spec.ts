/// <reference path="./@types/libp2p.ts" />

import assert from 'assert'
import Listener from './listener'
import Multiaddr from 'multiaddr'
import type { MultiaddrConnection, Upgrader } from 'libp2p'
import type { Connection } from 'libp2p'
import dgram from 'dgram'
import type { Socket, RemoteInfo } from 'dgram'
import { handleStunRequest } from './stun'
import PeerId from 'peer-id'
import net from 'net'
import Defer from 'p-defer'
import type { DeferredPromise } from 'p-defer'
import * as stun from 'webrtc-stun'

import { networkInterfaces } from 'os'

describe('check listening to sockets', function () {
  this.timeout(5000)

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

    const promises: Promise<void>[] = [
      new Promise((resolve) => socket.once('listening', resolve)),
      new Promise((resolve) => socket.bind(port, resolve))
    ]

    socket.on('message', (msg: Buffer, rinfo: RemoteInfo) => {
      state.msgReceived.resolve()
      handleStunRequest(socket, msg, rinfo)
    })

    await Promise.all(promises)

    return socket
  }

  async function stopStunServer(socket: Socket) {
    const promises: Promise<void>[] = [
      new Promise<void>((resolve) => socket.once('close', resolve)),
      new Promise<void>((resolve) => socket.close(resolve))
    ]

    await Promise.all(promises)
  }

  async function startBootstrap(state: { msgReceived: DeferredPromise<void> }) {
    const peerId = await PeerId.create({ keyType: 'secp256k1' })
    const listener = new Listener(
      undefined,
      ({
        upgradeInbound: async (conn: MultiaddrConnection) => {
          state.msgReceived.resolve()
          return conn
        },
        upgradeOutbound: async (conn: MultiaddrConnection) => conn
      } as unknown) as Upgrader,
      undefined,
      undefined,
      peerId,
      undefined
    )

    await listener.listen(Multiaddr(`/ip4/127.0.0.1/tcp/0`))

    return {
      peerId,
      listener
    }
  }

  async function stopListener(socket: Listener) {
    const promises: Promise<void>[] = [
      // prettier-ignore
      new Promise<void>((resolve) => socket.once('close', resolve)),
      socket.close()
    ]

    await Promise.all(promises)
  }

  async function waitUntilListening(socket: Listener, ma: Multiaddr) {
    const promises: Promise<void>[] = [
      new Promise<void>((resolve) => socket.once('listening', resolve)),
      socket.listen(ma)
    ]
    await Promise.all(promises)
  }

  it('recreate the socket and perform STUN request', async function () {
    let listener: Listener
    const peerId = await PeerId.create({ keyType: 'secp256k1' })
    // Create objects to pass boolean by reference and NOT by value
    const msgReceived = [
      {
        msgReceived: Defer<void>()
      },
      {
        msgReceived: Defer<void>()
      }
    ]

    const stunServers = [await startStunServer(9391, msgReceived[0]), await startStunServer(9392, msgReceived[1])]

    for (let i = 0; i < 2; i++) {
      listener = new Listener(
        () => {},
        (undefined as unknown) as Upgrader,
        [
          Multiaddr(`/ip4/127.0.0.1/udp/${stunServers[0].address().port}`),
          Multiaddr(`/ip4/127.0.0.1/udp/${stunServers[1].address().port}`)
        ],
        undefined,
        await PeerId.create({ keyType: 'secp256k1' }),
        undefined
      )

      await waitUntilListening(listener, Multiaddr(`/ip4/127.0.0.1/tcp/9390/p2p/${peerId.toB58String()}`))
      await stopListener(listener)
    }

    await Promise.all(msgReceived.map((received) => received.msgReceived.promise))

    await Promise.all(stunServers.map((s) => stopStunServer(s)))

    // await new Promise((resolve) => setTimeout(resolve, 200))
    assert(
      msgReceived[0].msgReceived && msgReceived[1].msgReceived,
      `Stun Server must have received messages from both Listener instances.`
    )
  })

  it('use relays and expose addrs', async function () {
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const relayContacted = {
      msgReceived: Defer<void>()
    }

    const bootstrap = await startBootstrap(relayContacted)

    const stunContacted = {
      msgReceived: Defer<void>()
    }

    const stunServer = await startStunServer(undefined, stunContacted)

    const listener = new Listener(
      () => {},
      ({
        upgradeOutbound: async (maConn: MultiaddrConnection) => maConn
      } as unknown) as Upgrader,
      [Multiaddr(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      [Multiaddr(`/ip4/127.0.0.1/tcp/${bootstrap.listener.getPort()}/p2p/${bootstrap.peerId.toB58String()}`)],
      peerId,
      undefined
    )

    await waitUntilListening(listener, Multiaddr(`/ip4/127.0.0.1/tcp/9390/p2p/${peerId.toB58String()}`))

    // Checks that relay and STUN got contacted, otherwise timeout
    await Promise.all([stunContacted.msgReceived.promise, relayContacted.msgReceived.promise])

    const listeningAddrs = listener.getAddrs().map((ma: Multiaddr) => ma.toString())

    assert(
      listeningAddrs.includes(`/p2p/${bootstrap.peerId.toB58String()}/p2p-circuit/p2p/${peerId.toB58String()}`),
      `Listener must expose circuit address`
    )

    await Promise.all([
      // prettier-ignore
      stopListener(listener),
      stopListener(bootstrap.listener),
      stopStunServer(stunServer)
    ])
  })

  it('should create two TCP sockets and exchange messages', async function () {
    const AMOUNT_OF_NODES = 2

    const ATTEMPTS = 5

    let msgReceived: { received: DeferredPromise<void> }[]

    const listeners = await Promise.all(
      Array.from({ length: AMOUNT_OF_NODES }).map(async (_, index) => {
        const peerId = await PeerId.create({ keyType: 'secp256k1' })

        const stunServers = []
        for (let i = 0; i < AMOUNT_OF_NODES; i++) {
          stunServers.push(Multiaddr(`/ip4/127.0.0.1/udp/${9390 + i}`))
        }

        const listener = new Listener(
          (conn: Connection) => {
            // @ts-ignore
            conn.conn.end()
            msgReceived[index].received.resolve()
          },
          ({
            upgradeInbound: async (conn: MultiaddrConnection) => conn
          } as unknown) as Upgrader,
          stunServers,
          undefined,
          await PeerId.create({ keyType: 'secp256k1' }),
          undefined
        )

        await waitUntilListening(listener, Multiaddr(`/ip6/::/tcp/${9390 + index}/p2p/${peerId.toB58String()}`))

        return listener
      })
    )

    for (let i = 0; i < ATTEMPTS; i++) {
      msgReceived = Array.from({ length: AMOUNT_OF_NODES }).map((_) => ({
        received: Defer()
      }))

      await Promise.all([
        new Promise<void>((resolve) => {
          const socket = net.createConnection(
            {
              host: '127.0.0.1',
              port: 9390 + (i % 2)
            },
            () => {
              socket.write(Buffer.from('test'), () => {
                socket.end()
                resolve()
              })
            }
          )
        }),
        new Promise<void>((resolve) => {
          const socket = net.createConnection(
            {
              host: '::1',
              port: 9390 + ((i + 1) % 2)
            },
            () => {
              socket.write(Buffer.from('test'), () => {
                socket.end()
                resolve()
              })
            }
          )
        }),
        ...msgReceived.map((received) => received.received.promise)
      ])
    }

    await Promise.all(listeners.map(stopListener))
  })

  it('should bind to specific interfaces', async function () {
    const validInterfaces = Object.keys(networkInterfaces()).filter((iface) =>
      networkInterfaces()[iface]?.some((x) => !x.internal)
    )

    if (validInterfaces.length == 0) {
      return
    }

    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const listener = new Listener(
      (conn: Connection) => {
        // @ts-ignore
        conn.conn.end()
      },
      ({
        upgradeInbound: async (conn: MultiaddrConnection) => conn
      } as unknown) as Upgrader,
      undefined,
      undefined,
      peerId,
      validInterfaces[0]
    )

    let errThrown = false
    try {
      await waitUntilListening(listener, Multiaddr(`/ip4/0.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))
    } catch {
      errThrown = true
    }

    assert(errThrown)

    await stopListener(listener)
  })

  it('should perform a STUN request', async function () {
    const defer = Defer<void>()
    const listener = new Listener(
      (conn: Connection) => {
        // @ts-ignore
        conn.conn.end()
      },
      ({
        upgradeInbound: async (conn: MultiaddrConnection) => conn
      } as unknown) as Upgrader,
      undefined,
      undefined,
      await PeerId.create({ keyType: 'secp256k1' }),
      undefined
    )

    await waitUntilListening(listener, Multiaddr(`/ip4/0.0.0.0/tcp/0`))

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

    const addrs = listener.getAddrs()

    const localAddress = addrs.find((ma: Multiaddr) => ma.toString().match(/127.0.0.1/))

    assert(localAddress != null, `Listener must be available on localhost`)

    socket.send(req.toBuffer(), localAddress.toOptions().port, `localhost`)

    await defer.promise

    await stopListener(listener)
  })

  // @TODO add test for connection tracking
})
