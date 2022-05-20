import Heartbeat, { type HeartbeatConfig, NetworkHealthIndicator } from './heartbeat'
import NetworkPeers from './network-peers'
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

class NetworkHealth extends EventEmitter {

  public state: NetworkHealthIndicator = NetworkHealthIndicator.UNKNOWN

  constructor() {
    super()
    this.on('hopr:network-health-changed', this.stateChanged.bind(this))
  }

  private stateChanged(_oldState: NetworkHealthIndicator, newState: NetworkHealthIndicator) {
    this.state = newState;
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

async function getPeer(
  self: PeerId,
  network: ReturnType<typeof createFakeNetwork>,
  netStatEvents: EventEmitter,
): Promise<{ heartbeat: TestingHeartbeat; peers: NetworkPeers }> {
  const peers = new NetworkPeers([], [self])

  const heartbeat = new TestingHeartbeat(
    peers,
    (protocol: string, handler: LibP2PHandlerFunction<any>) => network.subscribe(self, protocol, handler),
    ((dest: PeerId, protocol: string, msg: Uint8Array) => network.sendMessage(self, dest, protocol, msg)) as any,
    (async () => {
      assert.fail(`must not call hangUp`)
    }) as any,
    () => Promise.resolve(true),
    netStatEvents,
    (peerId) => peerId.toB58String() !== Charly.toB58String(),
    TESTING_ENVIRONMENT,
    {
      ...SHORT_TIMEOUTS,
      heartbeatThreshold: -3000,
      heartbeatInterval: 2000
    }
  )

  await heartbeat.start()

  return { heartbeat, peers }
}

describe('unit test heartbeat', async () => {
  it('check nodes is noop with empty store & health indicator is red', async () => {
    let netHealth = new NetworkHealth();
    const heartbeat = new TestingHeartbeat(
      new NetworkPeers([], [Alice]),
      (() => {}) as any,
      (async () => {
        assert.fail(`must not call send`)
      }) as any,
      (async () => {
        assert.fail(`must not call hangUp`)
      }) as any,
      () => Promise.resolve(true),
      netHealth,
      (_) => true,
      TESTING_ENVIRONMENT,
      SHORT_TIMEOUTS
    )
    await heartbeat.checkNodes()

    heartbeat.stop()

    assert.equal(netHealth.state, NetworkHealthIndicator.RED)
  })

  it('check network health state progression', async () => {
    const network = createFakeNetwork()

    const netHealthA = new NetworkHealth()

    const peerA = await getPeer(Alice, network, netHealthA)
    const peerB = await getPeer(Bob, network, new NetworkHealth())

    peerA.heartbeat.recalculateNetworkHealth()

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.2')
    assert.equal(netHealthA.state, NetworkHealthIndicator.RED)

    peerA.peers.register(Bob, 'test')

    peerA.heartbeat.recalculateNetworkHealth()

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.2')
    assert.equal(netHealthA.state, NetworkHealthIndicator.ORANGE)

    for (let i = 0; i < 4;i++) {
      await peerA.heartbeat.checkNodes()
    }

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.6')
    assert.equal(netHealthA.state, NetworkHealthIndicator.YELLOW)

    const peerC = await getPeer(Charly, network, new NetworkHealth())
    peerA.peers.register(Charly, 'test')

    assert.equal(netHealthA.state, NetworkHealthIndicator.YELLOW)

    for (let i = 0; i < 6;i++) {
      await peerA.heartbeat.checkNodes()
    }

    assert.equal(netHealthA.state, NetworkHealthIndicator.GREEN)

    // Losing private node Charly should take us back to yellow
    network.unsubscribe(Charly)
    peerC.heartbeat.stop()

    for (let i = 0; i < 10;i++) {
      await peerA.heartbeat.checkNodes()
    }

    assert.equal(netHealthA.state, NetworkHealthIndicator.YELLOW)

    ;[peerA, peerB].map((peer) => peer.heartbeat.stop())
    network.close()
  })

  it('check nodes does not change quality of newly registered peers', async () => {
    const network = createFakeNetwork()

    const peerA = await getPeer(Alice, network, new NetworkHealth())
    const peerB = await getPeer(Bob, network, new NetworkHealth())

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.2')

    peerA.peers.register(Bob, 'test')

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.2')

    await peerA.heartbeat.checkNodes()

    assert.equal(peerA.peers.qualityOf(Bob).toFixed(1), '0.3')
    ;[peerA, peerB].map((peer) => peer.heartbeat.stop())
    network.close()
  })

  it('check nodes does not change quality of offline peer', async () => {
    const network = createFakeNetwork()
    const peerA = await getPeer(Alice, network, new NetworkHealth())

    assert.equal(peerA.peers.qualityOf(Charly).toFixed(1), '0.2')

    peerA.peers.register(Charly, 'test')

    assert.equal(peerA.peers.qualityOf(Charly).toFixed(1), '0.2')

    await peerA.heartbeat.checkNodes()

    assert.equal(peerA.peers.qualityOf(Charly).toFixed(1), '0.2')

    peerA.heartbeat.stop()
    network.close()
  })

  it('test heartbeat flow', async () => {
    const network = createFakeNetwork()

    const peerA = await getPeer(Alice, network, new NetworkHealth())
    const peerB = await getPeer(Bob, network, new NetworkHealth())
    const peerC = await getPeer(Charly, network, new NetworkHealth())

    peerA.peers.register(Bob, 'test')
    peerA.peers.register(Charly, 'test')

    assert(peerA.peers.has(Charly), `Alice should know about Charly now.`)
    assert(peerA.peers.has(Bob), `Alice should know about Bob now.`)

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
