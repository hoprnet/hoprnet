import assert from 'assert'
import { Listener, MAX_RELAYS_PER_NODE } from './listener'
import { Multiaddr } from 'multiaddr'
import type { MultiaddrConnection, Upgrader } from 'libp2p-interfaces/transport'
import dgram, { type Socket } from 'dgram'
import PeerId from 'peer-id'
import { createConnection } from 'net'
import * as stun from 'webrtc-stun'
import { once, EventEmitter } from 'events'
import { randomBytes } from 'crypto'
import type { AddressInfo } from 'net'

import { type NetworkInterfaceInfo, networkInterfaces } from 'os'
import {
  u8aEquals,
  defer,
  type DeferType,
  toNetworkPrefix,
  u8aAddrToString,
  privKeyToPeerId,
  u8aToHex
} from '@hoprnet/hopr-utils'

import type { PublicNodesEmitter, PeerStoreType } from '../types'

import { waitUntilListening, stopNode, startStunServer } from './utils.spec'

/**
 * Decorated Listener class that allows access to
 * private class properties
 */
class TestingListener extends Listener {
  // @ts-ignore
  public uncheckedNodes: InstanceType<typeof Listener>['uncheckedNodes']
  // @ts-ignore
  public publicNodes: InstanceType<typeof Listener>['publicNodes']

  // @ts-ignore
  public addrs: InstanceType<typeof Listener>['addrs']

  // @ts-ignore
  public tcpSocket: InstanceType<typeof Listener>['tcpSocket']

  public onNewRelay(...args: Parameters<InstanceType<typeof Listener>['onNewRelay']>) {
    return super.onNewRelay(...args)
  }
  public onRemoveRelay(peer: PeerId) {
    return super.onRemoveRelay(peer)
  }

  public async updatePublicNodes() {
    return super.updatePublicNodes()
  }

  public getPort(): number {
    return (this.tcpSocket.address() as AddressInfo)?.port ?? -1
  }
}

function getPeerStoreEntry(addr: string, id = createPeerId()): PeerStoreType {
  return {
    id,
    multiaddrs: [new Multiaddr(addr)]
  }
}

/**
 * Synchronous function to sample PeerIds
 * @returns a PeerId
 */
function createPeerId(): PeerId {
  return privKeyToPeerId(u8aToHex(randomBytes(32)))
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
  initialNodes: PeerStoreType[] = [],
  state: { msgReceived?: DeferType<void>; expectedMessageReceived?: DeferType<void> } = {},
  expectedMessage: Uint8Array | undefined = undefined,
  peerId = createPeerId(),
  upgrader?: Upgrader,
  handler?: (conn: any) => any | Promise<any>,
  runningLocally?: boolean
) {
  const publicNodesEmitter = new EventEmitter() as PublicNodesEmitter

  const listener = new TestingListener(
    handler,
    upgrader ?? {
      upgradeInbound: async (conn: MultiaddrConnection) => {
        if (expectedMessage != undefined) {
          for await (const msg of conn.source) {
            if (u8aEquals(msg.slice(), expectedMessage)) {
              state?.expectedMessageReceived?.resolve()
            }
          }
        }

        state?.msgReceived?.resolve()
        return conn as any
      },
      upgradeOutbound: async (conn: MultiaddrConnection) => conn as any
    },
    publicNodesEmitter,
    initialNodes,
    peerId,
    undefined,
    runningLocally ?? false
  )

  await waitUntilListening(listener, new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${peerId.toB58String()}`))

  return {
    peerId,
    listener,
    publicNodesEmitter
  }
}

describe('check listening to sockets', function () {
  it('recreate the socket and perform STUN requests', async function () {
    this.timeout(10e3) // 3 seconds should be more than enough

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

    for (let i = 0; i < 3; i++) {
      listener = new TestingListener(
        undefined,
        undefined as any,
        undefined,
        [peerStoreEntries[i]],
        peerId,
        undefined,
        false
      )

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

  it('check that node is reachable', async function () {
    const stunServer = await startStunServer(undefined)
    const msgReceived = defer<void>()
    const expectedMessageReceived = defer<void>()

    const testMessage = new TextEncoder().encode('test')

    const node = await startNode(
      [getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
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
    const peerId = createPeerId()

    const listener = new Listener(
      undefined,
      {
        upgradeInbound: async (conn: MultiaddrConnection) => conn
      } as any,
      undefined,
      [getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)],
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

    const node = await startNode([getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)])

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

  it('check connection tracking', async function () {
    const stunServer = await startStunServer(undefined)
    const stunPeer = getPeerStoreEntry(`/ip4/127.0.0.1/udp/${stunServer.address().port}`)
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
})

describe('entry node functionality', function () {
  it('add public nodes', async function () {
    const node = await startNode()

    const peerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/0`)

    node.listener.onNewRelay(peerStoreEntry)
    // Should filter duplicate
    node.listener.onNewRelay(peerStoreEntry)

    assert(node.listener.uncheckedNodes.length == 1, `Unchecked nodes must contain one entry`)
    assert(node.listener.uncheckedNodes[0].id.equals(peerStoreEntry.id), `id must match the generated one`)
    assert(
      node.listener.uncheckedNodes[0].multiaddrs.length == peerStoreEntry.multiaddrs.length,
      `must not contain more multiaddrs`
    )

    assert(
      node.listener.addrs.relays == undefined || node.listener.addrs.relays.length == 0,
      `must not expose any internal addrs`
    )

    await stopNode(node.listener)
  })

  it('remove an offline node', async function () {
    const node = await startNode()

    const peerStoreEntry = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/0`)

    node.listener.publicNodes.push({
      ...peerStoreEntry,
      latency: 23
    })

    node.listener.onRemoveRelay(peerStoreEntry.id)

    assert(node.listener.publicNodes.length == 0, `must remove node from public nodes`)

    assert(
      node.listener.addrs.relays == undefined || node.listener.addrs.relays.length == 0,
      `must not expose any internal addrs`
    )

    await stopNode(node.listener)
  })

  it('contact potential relays and update relay addresses', async function () {
    const relayContacted = defer<void>()

    const relay = await startNode(undefined, {
      msgReceived: relayContacted
    })

    const node = await startNode([getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}`, relay.peerId)])

    await relayContacted.promise

    assert(node.listener.publicNodes.length == 1, `must contain exactly one public node`)
    assert(node.listener.publicNodes[0].id.equals(relay.peerId), `must contain correct peerId`)
    assert(node.listener.publicNodes[0].latency >= 0, `latency must be non-negative`)

    assert(node.listener.addrs.relays != undefined, `must expose relay addrs`)
    assert(node.listener.addrs.relays.length == 1, `must expose exactly one relay addrs`)
    assert(
      node.listener.addrs.relays[0].toString() ===
        `/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`,
      `must expose the right relay address`
    )

    await Promise.all([stopNode(relay.listener), stopNode(node.listener)])
  })

  it('expose limited number of relay addresses', async function () {
    this.timeout(10e3)

    const relayNodes = await Promise.all(
      Array.from<any, Promise<[DeferType<void>, Awaited<ReturnType<typeof startNode>>]>>(
        { length: MAX_RELAYS_PER_NODE },
        async () => {
          const relayContacted = defer<void>()

          const relay = await startNode(undefined, {
            msgReceived: relayContacted
          })

          return [relayContacted, relay]
        }
      )
    )

    const node = await startNode(
      relayNodes.map((relayNode) =>
        getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${relayNode[1].listener.getPort()}`, relayNode[1].peerId)
      )
    )

    await Promise.all(relayNodes.map((relayNode) => relayNode[0].promise))

    assert(node.listener.addrs.relays != undefined, `must expose relay addresses`)
    assert(
      node.listener.addrs.relays.length == MAX_RELAYS_PER_NODE,
      `must expose ${MAX_RELAYS_PER_NODE} relay addresses`
    )

    assert(
      relayNodes.every((relayNode) =>
        node.listener.publicNodes.some((publicNode) => publicNode.id.equals(relayNode[1].peerId))
      ),
      `must contain all relay nodes`
    )

    await Promise.all(
      relayNodes
        .map((relayNode) => relayNode[1])
        .concat(node)
        .map((node) => stopNode(node.listener))
    )
  })

  it('update nodes once node became offline', async function () {
    const node = await startNode()

    const newNode = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/12345`)

    const relayContacted = defer<void>()

    const relay = await startNode(undefined, {
      msgReceived: relayContacted
    })

    node.listener.publicNodes.push({
      ...newNode,
      latency: 23
    })

    node.listener.addrs.relays.push(
      new Multiaddr(`/p2p/${newNode.id.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`)
    )

    assert(node.listener.addrs.relays.length == 1)
    assert(node.listener.publicNodes.length == 1)

    node.listener.uncheckedNodes.push(getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}`, relay.peerId))

    const listeningEventPromise = once(node.listener, 'listening')
    node.listener.onRemoveRelay(newNode.id)

    // @ts-ignore
    assert(node.listener.publicNodes.length == 0)

    await relayContacted.promise
    await listeningEventPromise

    assert(node.listener.publicNodes.length == 1)

    await Promise.all([stopNode(node.listener), stopNode(relay.listener)])
  })

  it('take those nodes that are online', async function () {
    const node = await startNode()

    const relayContacted = defer<void>()

    const relay = await startNode(undefined, {
      msgReceived: relayContacted
    })

    const fakeNode = getPeerStoreEntry(`/ip4/127.0.0.1/tcp/12345`)

    node.listener.uncheckedNodes.push(getPeerStoreEntry(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}`, relay.peerId))
    node.listener.uncheckedNodes.push(fakeNode)

    const listeningEventPromise = once(node.listener, 'listening')
    node.listener.updatePublicNodes()

    await relayContacted.promise
    await listeningEventPromise

    assert(node.listener.publicNodes.length == 1)
    assert(node.listener.publicNodes[0].id.equals(relay.peerId))

    await Promise.all([stopNode(node.listener), stopNode(relay.listener)])
  })

  it('do not emit listening event if nothing has changed', async function () {
    const node = await startNode()

    const relay = await startNode()

    node.listener.publicNodes.push({
      id: relay.peerId,
      multiaddrs: [new Multiaddr(`/ip4/127.0.0.1/tcp/${relay.listener.getPort()}/p2p/${relay.peerId.toB58String()}`)],
      latency: 23
    })
    node.listener.addrs.relays.push(
      new Multiaddr(`/p2p/${relay.peerId.toB58String()}/p2p-circuit/p2p/${node.peerId.toB58String()}`)
    )

    node.listener.once('listening', () =>
      assert.fail(`must not throw listening event if list of entry nodes has not changed`)
    )

    await node.listener.updatePublicNodes()

    assert(node.listener.publicNodes.length == 1)
    assert(node.listener.publicNodes[0].id.equals(relay.peerId))

    assert(node.listener.addrs)

    await Promise.all([stopNode(node.listener), stopNode(relay.listener)])
  })
})

describe('error cases', function () {
  it('throw error while upgrading the connection', async () => {
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
