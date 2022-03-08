import Heartbeat, { type HeartbeatConfig } from './heartbeat'
import NetworkPeerStore from './network-peers'
import { assert } from 'chai'
import { type LibP2PHandlerFunction, privKeyToPeerId } from '@hoprnet/hopr-utils'
import { EventEmitter, once } from 'events'
import type PeerId from 'peer-id'
import { NETWORK_QUALITY_THRESHOLD } from '../constants'

class TestingHeartbeat extends Heartbeat {
  public async checkNodes() {
    return await super.checkNodes()
  }
}

const Alice = privKeyToPeerId('0x427ff36aacbac09f6da4072161a6a338308c53cfb6e50ca56aa70b1a38602a9f')
const Bob = privKeyToPeerId('0xf9bfbad938482b29076932b080fb6ac1e14616ee621fb3f77739784bcf1ee8cf')
const Charly = privKeyToPeerId('0xfab2610822e8c973bec74c811e2f44b6b4b501e922b1d67f5367a26ce46088ea')

const TESTING_ENVIRONMENT = 'unit-testing'

// Overwrite default timeouts with shorter ones for unit testing
const SHORT_TIMEOUTS: Partial<HeartbeatConfig> = {
  heartbeatDialTimeout: 50,
  heartbeatRunTimeout: 100,
  heartbeatInterval: 200,
  heartbeatVariance: 1
}

/**
 * Used to mock sending messages using events
 * @param self peerId of the destination
 * @param protocol protocol to speak with receiver
 * @returns an event string that includes destination and protocol
 */
function reqEventName(self: PeerId, protocol: string): string {
  return `req:${self.toB58String()}:${protocol}`
}

/**
 * Used to mock replying to incoming messages
 * @param self peerId of the sender
 * @param dest peerId of the destination
 * @param protocol protocol to speak with receiver
 * @returns an event string that includes sender, receiver and the protocol
 */
function resEventName(self: PeerId, dest: PeerId, protocol: string): string {
  return `res:${self.toB58String()}:${dest.toB58String()}:${protocol}`
}
/**
 * Creates an event-based fake network
 * @returns a fake network
 */
function createFakeNetwork() {
  const network = new EventEmitter()

  const subscribedPeers = new Map<string, string>()

  // mocks libp2p.handle(protocol)
  const subscribe = (
    self: PeerId,
    protocol: string,
    handler: (msg: Uint8Array, remotePeer: PeerId) => Promise<Uint8Array>
  ) => {
    network.on(reqEventName(self, protocol), async (from: PeerId, request: Uint8Array) => {
      const response = await handler(request, from)

      network.emit(resEventName(self, from, protocol), self, response)
    })

    subscribedPeers.set(self.toB58String(), reqEventName(self, protocol))
  }

  // mocks libp2p.dialProtocol
  const sendMessage = async (self: PeerId, dest: PeerId, protocol: string, msg: Uint8Array) => {
    if (network.listenerCount(reqEventName(dest, protocol)) > 0) {
      const recvPromise = once(network, resEventName(dest, self, protocol))

      network.emit(reqEventName(dest, protocol), self, msg)

      const result = (await recvPromise) as [from: PeerId, response: Uint8Array]

      return Promise.resolve([result[1]])
    }

    return Promise.reject()
  }

  // mocks libp2p.stop
  const unsubscribe = (peer: PeerId) => {
    if (subscribedPeers.has(peer.toB58String())) {
      const protocol = subscribedPeers.get(peer.toB58String())

      network.removeAllListeners(protocol)
    }
  }

  return {
    subscribe,
    sendMessage,
    close: network.removeAllListeners.bind(network),
    unsubscribe
  }
}

function getPeer(self: PeerId, network: ReturnType<typeof createFakeNetwork>) {
  const peers = new NetworkPeerStore([], [self])

  const heartbeat = new TestingHeartbeat(
    peers,
    (protocol: string, handler: LibP2PHandlerFunction<any>) => network.subscribe(self, protocol, handler),
    ((dest: PeerId, protocol: string, msg: Uint8Array) => network.sendMessage(self, dest, protocol, msg)) as any,
    (async () => {
      assert.fail(`must not call hangUp`)
    }) as any,
    TESTING_ENVIRONMENT,
    {
      ...SHORT_TIMEOUTS,
      heartbeatThreshold: -3000,
      heartbeatInterval: 2000
    }
  )

  heartbeat.start()

  return { heartbeat, peers }
}

describe('unit test heartbeat', async () => {
  it('check nodes is noop with empty store', async () => {
    const heartbeat = new TestingHeartbeat(
      new NetworkPeerStore([], [Alice]),
      (() => {}) as any,
      (async () => {
        assert.fail(`must not call send`)
      }) as any,
      (async () => {
        assert.fail(`must not call hangUp`)
      }) as any,
      TESTING_ENVIRONMENT,
      SHORT_TIMEOUTS
    )
    await heartbeat.checkNodes()

    heartbeat.stop()
  })

  it('check nodes does not change quality of newly registered peers', async () => {
    const network = createFakeNetwork()
    const peerA = getPeer(Alice, network)

    const peerB = getPeer(Bob, network)

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.2')

    peerA.peers.register(Bob)

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.2')

    await peerA.heartbeat.checkNodes()

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.2')
    ;[peerA, peerB].map((peer) => peer.heartbeat.stop())
    network.close()
  })

  it('check nodes does not change quality of offline peer', async () => {
    const network = createFakeNetwork()
    const peerA = getPeer(Alice, network)

    assert.equal(peerA.peers.qualityOf(Charly).toFixed(1), '0.2')

    peerA.peers.register(Charly)

    assert.equal(peerA.peers.qualityOf(Charly).toFixed(1), '0.2')

    await peerA.heartbeat.checkNodes()

    assert.equal(peerA.peers.qualityOf(Charly).toFixed(1), '0.2')

    peerA.heartbeat.stop()
    network.close()
  })

  it('test heartbeat flow', async () => {
    const network = createFakeNetwork()

    const peerA = getPeer(Alice, network)
    const peerB = getPeer(Bob, network)
    const peerC = getPeer(Charly, network)

    peerA.peers.register(Bob)
    peerA.peers.register(Charly)

    assert(peerA.peers.has(Charly), `Alice should know about Charly now.`)
    assert(peerA.peers.has(Bob), `Alice should know about Bob now.`)

    await peerA.heartbeat.checkNodes()
    await peerA.heartbeat.checkNodes()
    await peerA.heartbeat.checkNodes()
    await peerA.heartbeat.checkNodes()
    await peerA.heartbeat.checkNodes()

    assert.isAbove(peerA.peers.qualityOf(Bob), NETWORK_QUALITY_THRESHOLD)
    assert.isAbove(peerA.peers.qualityOf(Charly), NETWORK_QUALITY_THRESHOLD)

    network.unsubscribe(Charly)
    peerC.heartbeat.stop()

    await peerA.heartbeat.checkNodes()
    await peerA.heartbeat.checkNodes()

    assert.isAbove(peerA.peers.qualityOf(Bob), NETWORK_QUALITY_THRESHOLD)
    assert.isAtMost(peerA.peers.qualityOf(Charly), NETWORK_QUALITY_THRESHOLD)

    peerA.heartbeat.stop()
    peerB.heartbeat.stop()
  })
})
