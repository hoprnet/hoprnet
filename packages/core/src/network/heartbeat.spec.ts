// import Heartbeat from './heartbeat.js'
// import { HeartbeatConfig, Health, Network, PeerOrigin } from '../../lib/core_network.js'
// import { assert } from 'chai'
// import { privKeyToPeerId } from '@hoprnet/hopr-utils'
// import { EventEmitter, once } from 'events'
// import type { PeerId } from '@libp2p/interface-peer-id'
// import { MAX_PARALLEL_PINGS, NETWORK_QUALITY_THRESHOLD } from '../constants.js'
// import type { Connection, Stream } from '@libp2p/interfaces/connection'
// import type { Components } from '@libp2p/interfaces/components'
// import { peerIdFromString } from '@libp2p/peer-id'

// class TestingHeartbeat extends Heartbeat {
//   public async checkNodes() {
//     try {
//       // timestamp far in the future to allow pinging immediately after the previous ping in the tests
//       const thresholdTime = Date.now() + 1_000_000

//       return await this.pinger.ping(this.networkPeers.peers_to_ping(BigInt(thresholdTime)))
//     } catch (err) {
//       assert.fail('Should not be thrown')
//     }
//   }
// }

// const Me = privKeyToPeerId('0x9135f358f94b59e8cdee5545eb9ecc8ff32bc3a79227a09ee2bb6b50f1ad8159')
// const Alice = privKeyToPeerId('0x427ff36aacbac09f6da4072161a6a338308c53cfb6e50ca56aa70b1a38602a9f')
// const Bob = privKeyToPeerId('0xf9bfbad938482b29076932b080fb6ac1e14616ee621fb3f77739784bcf1ee8cf')
// const Charly = privKeyToPeerId('0xfab2610822e8c973bec74c811e2f44b6b4b501e922b1d67f5367a26ce46088ea')

// const TESTING_ENVIRONMENT = 'unit-testing'

// /**
//  * Used to mock sending messages using events
//  * @param self peerId of the destination
//  * @param protocol protocol to speak with receiver
//  * @returns an event string that includes destination and protocol
//  */
// function reqEventName(self: PeerId, protocol: string): string {
//   return `req:${self.toString()}:${protocol}`
// }

// /**
//  * Used to mock replying to incoming messages
//  * @param self peerId of the sender
//  * @param dest peerId of the destination
//  * @param protocol protocol to speak with receiver
//  * @returns an event string that includes sender, receiver and the protocol
//  */
// function resEventName(self: PeerId, dest: PeerId, protocol: string): string {
//   return `res:${self.toString()}:${dest.toString()}:${protocol}`
// }

// /**
//  * Creates an event-based fake network
//  * @returns a fake network
//  */
// function createFakeNetwork() {
//   const network = new EventEmitter()

//   const subscribedPeers = new Map<string, string>()

//   // mocks libp2p.handle(protocol)
//   const subscribe = (
//     self: PeerId,
//     protocols: string | string[],
//     handler: ({
//       connection,
//       stream,
//       protocol
//     }: {
//       connection: Connection
//       stream: Stream
//       protocol: string
//     }) => Promise<void>
//   ) => {
//     let protocol: string
//     if (Array.isArray(protocols)) {
//       protocol = protocols[0]
//     } else {
//       protocol = protocols
//     }

//     network.on(reqEventName(self, protocol), async (from: PeerId, request: Uint8Array) => {
//       await handler({
//         connection: {
//           remotePeer: from
//         } as Connection,
//         protocol,
//         stream: {
//           // Assuming that we are receiving exactly one message, namely ping
//           source: (async function* () {
//             yield request
//           })(),
//           sink: async function (source: AsyncIterable<Uint8Array>) {
//             for await (const msg of source) {
//               // Assuming that we are sending exactly one message, namely pong
//               network.emit(resEventName(self, from, protocol), self, msg)
//               break
//             }
//           }
//         } as any
//       })
//     })

//     subscribedPeers.set(self.toString(), reqEventName(self, protocol))
//   }

//   // mocks libp2p.dialProtocol
//   const sendMessage = async (
//     self: PeerId,
//     dest: PeerId,
//     protocols: string | string[],
//     msg: Uint8Array
//   ): Promise<Uint8Array[]> => {
//     let protocol: string
//     if (Array.isArray(protocols)) {
//       protocol = protocols[0]
//     } else {
//       protocol = protocols
//     }

//     if (network.listenerCount(reqEventName(dest, protocol)) > 0) {
//       const recvPromise = once(network, resEventName(dest, self, protocol))

//       network.emit(reqEventName(dest, protocol), self, msg)

//       const result = (await recvPromise) as [from: PeerId, response: Uint8Array]

//       return Promise.resolve([result[1]])
//     }

//     return Promise.reject()
//   }

//   // mocks libp2p.stop
//   const unsubscribe = (peer: PeerId) => {
//     if (subscribedPeers.has(peer.toString())) {
//       const protocol = subscribedPeers.get(peer.toString())

//       network.removeAllListeners(protocol)
//     }
//   }

//   return {
//     subscribe,
//     sendMessage,
//     close: network.removeAllListeners.bind(network),
//     unsubscribe
//   }
// }

// async function getPeer(
//   self: PeerId,
//   network: ReturnType<typeof createFakeNetwork>
// ): Promise<{ heartbeat: TestingHeartbeat; peers: Network }> {
//   const peers = Network.build(
//     self.toString(),
//     NETWORK_QUALITY_THRESHOLD,
//     (_: string) => {},
//     (_o: Health, _n: Health) => {},
//     (peerId: string) => !peerIdFromString(peerId).equals(Charly) && !peerIdFromString(peerId).equals(Me),
//     (async () => {
//       assert.fail(`must not call hangUp`)
//     }) as any
//   )

//   let cfg = HeartbeatConfig.build(MAX_PARALLEL_PINGS, 1, 2000, BigInt(15000))

//   const heartbeat = new TestingHeartbeat(
//     peers,
//     {
//       getRegistrar() {
//         return {
//           async handle(
//             protocols: string | string[],
//             handler: ({
//               protocol,
//               stream,
//               connection
//             }: {
//               protocol: string
//               stream: Stream
//               connection: Connection
//             }) => Promise<void>
//           ) {
//             network.subscribe(self, protocols, handler)
//           }
//         } as NonNullable<Components['registrar']>
//       }
//     } as Components,
//     ((dest: PeerId, protocols: string | string[], msg: Uint8Array) =>
//       network.sendMessage(self, dest, protocols, msg)) as any,
//     TESTING_ENVIRONMENT,
//     cfg
//   )

//   await heartbeat.start()

//   return { heartbeat, peers }
// }

// describe('integration test heartbeat', async () => {
//   it('test heartbeat flow with multiple peers', async () => {
//     const network = createFakeNetwork()

//     const peerA = await getPeer(Alice, network)
//     const peerB = await getPeer(Bob, network)
//     const peerC = await getPeer(Charly, network)

//     peerA.peers.register(Bob.toString(), PeerOrigin.Initialization)
//     peerA.peers.register(Charly.toString(), PeerOrigin.Initialization)

//     assert(peerA.peers.contains(Charly.toString()), `Alice should know about Charly now.`)
//     assert(peerA.peers.contains(Bob.toString()), `Alice should know about Bob now.`)

//     await peerA.heartbeat.checkNodes()
//     await peerA.heartbeat.checkNodes()
//     await peerA.heartbeat.checkNodes()
//     await peerA.heartbeat.checkNodes()
//     await peerA.heartbeat.checkNodes()

//     assert.equal(peerA.peers.quality_of(Bob.toString()), NETWORK_QUALITY_THRESHOLD)
//     assert.equal(peerA.peers.quality_of(Charly.toString()), NETWORK_QUALITY_THRESHOLD)

//     network.unsubscribe(Charly)
//     peerC.heartbeat.stop()

//     await peerA.heartbeat.checkNodes()
//     await peerA.heartbeat.checkNodes()

//     assert.isAbove(peerA.peers.quality_of(Bob.toString()), NETWORK_QUALITY_THRESHOLD)
//     assert.isBelow(peerA.peers.quality_of(Charly.toString()), NETWORK_QUALITY_THRESHOLD)

//     peerA.heartbeat.stop()
//     peerB.heartbeat.stop()
//   })
// })
