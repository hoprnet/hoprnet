import assert from 'assert'
import { Listener } from './listener'
import { Multiaddr } from 'multiaddr'
import type { MultiaddrConnection } from 'libp2p-interfaces/src/transport/types'
import type Connection from 'libp2p-interfaces/src/connection/connection'
import dgram, { type Socket } from 'dgram'
import { createConnection, type AddressInfo } from 'net'
import * as stun from 'webrtc-stun'
import { once, EventEmitter } from 'events'

import { type NetworkInterfaceInfo, networkInterfaces } from 'os'
import { u8aEquals, defer, type DeferType, toNetworkPrefix, u8aAddrToString } from '@hoprnet/hopr-utils'

import type { PublicNodesEmitter, PeerStoreType, HoprConnectTestingOptions } from '../types'

import { stopNode, startStunServer, getPeerStoreEntry, createPeerId } from './utils.spec'

/**
 * Decorated Listener class that allows access to
 * private class properties
 */
class TestingListener extends Listener {
  // @ts-ignore
  public addrs: InstanceType<typeof Listener>['addrs']

  // @ts-ignore
  public tcpSocket: InstanceType<typeof Listener>['tcpSocket']

  // @ts-ignore
  public __connections: InstanceType<typeof Listener>['__connections']
  /**
   * Get amount of currently open connections
   * @dev used for testing
   * @returns amount of currently open connections
   */
  getConnections(): number {
    return this.__connections.length
  }

  public getPort(): number {
    return (this.tcpSocket.address() as AddressInfo)?.port ?? -1
  }
}

const localHostBeingExposed: HoprConnectTestingOptions = {
  __runningLocally: true
}

const localHostCheckingNAT: HoprConnectTestingOptions = {
  __noUPNP: true,
  __runningLocally: false, // contact STUN servers
  __preferLocalAddresses: true // accept local addresses from STUN servers
}

/**
 * Creates a node and attaches message listener to it.
 * @param publicNodes emitter that emit an event on new public nodes
 * @param state check message reception and content of message
 * @param expectedMessage message to check for, or undefined to skip this check
 * @param peerId peerId of the node
 * @returns
 */
async function startNode(
  initialNodes: PeerStoreType[] = [],
  state: { msgReceived?: DeferType<void>; expectedMessageReceived?: DeferType<void> } = {},
  expectedMessage: Uint8Array | undefined = undefined,
  peerId = createPeerId(),
  upgradeInbound: ((maConn: MultiaddrConnection) => Promise<Connection>) | undefined
) {
  const publicNodes = new EventEmitter() as PublicNodesEmitter

  const listener = new TestingListener(
    (async () => {}) as any,
    upgradeInbound ??
      (async (conn: MultiaddrConnection) => {
        if (expectedMessage != undefined) {
          for await (const msg of conn.source) {
            if (u8aEquals(msg.slice(), expectedMessage)) {
              state?.expectedMessageReceived?.resolve()
            }
          }
        }

        state?.msgReceived?.resolve()
        return conn as any
      }),
    peerId,
    {
      publicNodes,
      initialNodes
    },
    localHostCheckingNAT,
    {
      setAddrs: () => {}
    } as any,
    {
      setUsedRelays: () => {}
    } as any
  )

  await listener.listen(new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))

  return {
    peerId,
    listener,
    publicNodesEmitter: publicNodes
  }
}

describe('check listening to sockets', function () {
  it('recreate the socket and perform STUN requests', async function () {
    this.timeout(10e3) // 3 seconds should be more than enough
    const secondStunServer = await startStunServer(undefined)

    let listener: TestingListener
    const peerId = createPeerId()

    const AMOUNT = 3

    const msgReceived = Array.from({ length: AMOUNT }, (_) => defer<void>())

    const stunServers = await Promise.all(
      Array.from({ length: AMOUNT }, (_, index: number) =>
        startStunServer(undefined, { msgReceived: msgReceived[index] })
      )
    )

    const peerStoreEntries = stunServers.map((s: Socket) => getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${s.address().port}`))

    let port: number | undefined

    for (const peerStoreEntry of peerStoreEntries) {
      listener = new TestingListener(
        (async () => {}) as any,
        undefined as any,
        peerId,
        {
          initialNodes: [peerStoreEntry, getPeerStoreEntry(`/ip4/127.0.0.1/udp/${secondStunServer.address().port}`)]
        },
        localHostCheckingNAT,
        {
          setAddrs: () => {}
        } as any,
        {
          setUsedRelays: () => {}
        } as any
      )

      let listeningMultiaddr: Multiaddr
      if (port != undefined) {
        listeningMultiaddr = new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`)
      } else {
        // Listen to previously used port
        listeningMultiaddr = new Multiaddr(`/ip4/127.0.0.1/tcp/${port}/p2p/${peerId.toB58String()}`)
      }

      await listener.listen(listeningMultiaddr)

      if (port == undefined) {
        // Store the port to which we have listened before
        port = listener.getPort()
      }
      assert(port != undefined)
      await stopNode(listener)
    }

    await Promise.all(msgReceived.map((received) => received.promise))
    await Promise.all(stunServers.concat(secondStunServer).map(stopNode))
  })

  it('check that node is reachable', async function () {
    const firstStunServer = await startStunServer(undefined)
    const secondStunServer = await startStunServer(undefined)

    const msgReceived = defer<void>()
    const expectedMessageReceived = defer<void>()

    const testMessage = new TextEncoder().encode('test')

    const node = await startNode(
      [
        getPeerStoreEntry(`/ip4/127.0.0.1/udp/${firstStunServer.address().port}`),
        getPeerStoreEntry(`/ip4/127.0.0.1/udp/${secondStunServer.address().port}`)
      ],
      {
        msgReceived,
        expectedMessageReceived
      },
      testMessage,
      undefined,
      undefined
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

    // Produces a timeout if not successful
    await Promise.all([msgReceived.promise, expectedMessageReceived.promise])

    await Promise.all([node.listener, firstStunServer, secondStunServer].map(stopNode))
  })

  it('should bind to specific interfaces', async function () {
    // Test does do not do anything if there are only IPv6 addresses
    const usableInterfaces = networkInterfaces()

    for (const iface of Object.keys(usableInterfaces)) {
      const osIface = usableInterfaces[iface]

      // Disable IPv6
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

      // Disable IPv6
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
    const peerId = createPeerId()

    const listener = new Listener(
      (async () => {}) as any,
      {
        upgradeInbound: async (conn: MultiaddrConnection) => conn
      } as any,
      peerId,
      {
        interface: firstUsableInterfaceName,
        initialNodes: [getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)]
      },
      localHostBeingExposed,
      {
        setAddrs: () => {}
      } as any,
      {
        setUsedRelays: () => {}
      } as any
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
    const firstStunServer = await startStunServer(undefined)
    const secondStunServer = await startStunServer(undefined)

    const node = await startNode(
      [
        getPeerStoreEntry(`/ip4/127.0.0.1/udp/${firstStunServer.address().port}`),
        getPeerStoreEntry(`/ip4/127.0.0.1/udp/${secondStunServer.address().port}`)
      ],
      undefined,
      undefined,
      undefined,
      undefined
    )

    const stunServerSocket = dgram.createSocket({ type: 'udp4' })
    const tid = stun.generateTransactionId()

    stunServerSocket.on('message', (msg) => {
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
    })

    const req = stun.createBindingRequest(tid).setFingerprintAttribute()

    const addrs = node.listener.getAddrs()

    const localAddress = addrs.find((ma: Multiaddr) => ma.toString().match(/127.0.0.1/))

    assert(localAddress != null, `Listener must be available on localhost`)

    stunServerSocket.send(req.toBuffer(), localAddress.toOptions().port, `localhost`)

    await msgReceived.promise

    stunServerSocket.close()

    await Promise.all([node.listener, firstStunServer, secondStunServer].map(stopNode))
  })

  it('check connection tracking', async function () {
    const firstStunServer = await startStunServer(undefined)
    const secondStunServer = await startStunServer(undefined)
    const msgReceived = defer<void>()
    const expectedMessageReceived = defer<void>()

    const node = await startNode(
      [
        getPeerStoreEntry(`/ip4/127.0.0.1/udp/${firstStunServer.address().port}`),
        getPeerStoreEntry(`/ip4/127.0.0.1/udp/${secondStunServer.address().port}`)
      ],
      {
        msgReceived,
        expectedMessageReceived
      },
      undefined,
      undefined,
      undefined
    )

    const firstConnectionDone = defer<void>()
    const secondConnectionDone = defer<void>()

    const socketOne = createConnection(
      {
        host: '127.0.0.1',
        port: node.listener.getPort()
      },
      firstConnectionDone.resolve.bind(firstConnectionDone)
    )

    const socketTwo = createConnection(
      {
        host: '127.0.0.1',
        port: node.listener.getPort()
      },
      secondConnectionDone.resolve.bind(secondConnectionDone)
    )

    await Promise.all([firstConnectionDone.promise, secondConnectionDone.promise])

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

    await Promise.all([node.listener, firstStunServer, secondStunServer].map(stopNode))
  })

  it('determine NAT situation', async function () {
    const firstStunServer = await startStunServer(undefined)
    const secondStunServer = await startStunServer(undefined)

    const listener = new Listener(
      (async () => {}) as any,
      (() => {}) as any,
      createPeerId(),
      {
        initialNodes: [
          getPeerStoreEntry(`/ip4/127.0.0.1/udp/${firstStunServer.address().port}`),
          getPeerStoreEntry(`/ip4/127.0.0.1/udp/${secondStunServer.address().port}`)
        ]
      },
      localHostCheckingNAT,
      {
        setAddrs: () => {}
      } as any,
      {
        setUsedRelays: () => {}
      } as any
    )
    await listener.bind(new Multiaddr(`/ip4/0.0.0.0/tcp/9091`))
    await assert.doesNotReject(async () => await listener.checkNATSituation(`127.0.0.1`, 9091))
    await Promise.all([listener, firstStunServer, secondStunServer].map(stopNode))
  })

  it('determine NAT situation in localMode', async function () {
    const firstStunServer = await startStunServer(undefined)
    const secondStunServer = await startStunServer(undefined)

    const listener = new Listener(
      (async () => {}) as any,
      (() => {}) as any,
      createPeerId(),
      {
        initialNodes: [
          getPeerStoreEntry(`/ip4/127.0.0.1/udp/${firstStunServer.address().port}`),
          getPeerStoreEntry(`/ip4/127.0.0.1/udp/${secondStunServer.address().port}`)
        ]
      },
      localHostBeingExposed,
      {
        setAddrs: () => {}
      } as any,
      {
        setUsedRelays: () => {}
      } as any
    )

    await listener.bind(new Multiaddr(`/ip4/0.0.0.0/tcp/9091`))
    const natResult = await listener.checkNATSituation(`127.0.0.1`, 9091)

    assert(natResult.bidirectionalNAT === false)
    assert(['::', '0.0.0.0'].includes(natResult.externalAddress))
    assert(Number.isInteger(natResult.externalPort))
    assert(natResult.isExposed === true)

    await Promise.all([listener, firstStunServer, secondStunServer].map(stopNode))
  })
})

describe('error cases', function () {
  it('throw error while upgrading the connection', async () => {
    this.timeout(10e3)
    const peer = createPeerId()
    const firstStunServer = await startStunServer(undefined)
    const secondStunServer = await startStunServer(undefined)

    const node = await startNode(
      [
        getPeerStoreEntry(`/ip4/127.0.0.1/udp/${firstStunServer.address().port}`),
        getPeerStoreEntry(`/ip4/127.0.0.1/udp/${secondStunServer.address().port}`)
      ],
      undefined,
      undefined,
      peer,
      (() => {
        throw Error()
      }) as any
    )

    const connectionEstablished = defer<void>()
    const socket = createConnection(
      {
        host: '127.0.0.1',
        port: node.listener.getPort()
      },
      async () => {
        await new Promise((resolve) => setTimeout(resolve, 200))

        connectionEstablished.resolve()

        socket.end()
      }
    )

    await connectionEstablished.promise

    socket.destroy()

    await Promise.all([node.listener, firstStunServer, secondStunServer].map(stopNode))
  })

  it('throw unexpected error', async function () {
    // This unit test case produces an uncaught error in case there
    // is no "global" try / catch on incoming socket connections
    const peer = createPeerId()
    const stunServer = await startStunServer(undefined)

    const node = await startNode(
      [getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
      undefined,
      undefined,
      peer,
      {
        upgradeInbound: async (_maConn: MultiaddrConnection) => {
          await new Promise((resolve) => setTimeout(resolve, 100))

          // Do sth. unexpected
          // @ts-ignore
          conn.nonExisting()

          return {}
        }
      } as any
    )

    const connectionEstablished = defer<void>()
    const socket = createConnection(
      {
        host: '127.0.0.1',
        port: node.listener.getPort()
      },
      async () => {
        await new Promise((resolve) => setTimeout(resolve, 200))

        connectionEstablished.resolve()

        socket.end()
      }
    )

    await connectionEstablished.promise

    await Promise.all([node.listener, stunServer].map(stopNode))
  })
})
