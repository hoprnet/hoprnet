// import type { HoprConnectTestingOptions, StreamType } from '../types.js'
// import type { StreamHandler } from '@libp2p/interfaces/registrar'
// import type { Connection } from '@libp2p/interface-connection'
// import type { Address } from '@libp2p/interface-peer-store'
// import type { Components } from '@libp2p/interfaces/components'
// import type { PeerId } from '@libp2p/interface-peer-id'

// import { peerIdFromString } from '@libp2p/peer-id'
// import { pair } from 'it-pair'
// import { handshake } from 'it-handshake'
// import { Multiaddr } from '@multiformats/multiaddr'

// import EventEmitter from 'events'
// import assert from 'assert'

// import { Relay } from './index.js'
// import { privKeyToPeerId, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
// import type { RelayConnection } from './connection.js'
// import type { ConnectComponents } from '../components.js'

// const initiator = privKeyToPeerId(stringToU8a('0xa889bad3e2a31cceff4faccdd374af67db485ac0e05e7e654530aff0da5199f7'))
// const relay = privKeyToPeerId(stringToU8a('0xcd1fb76053833d9bb5b3ff243b2d17b96dc5ad7cc09b33c4cf77ba83c297443f'))
// const counterparty = privKeyToPeerId(stringToU8a('0x4090ca3740b1fe0f6da22befc4f7cba26389c51808d245dd29a2076fc66103aa'))

// function msgToEchoedMessage(message: string): Uint8Array {
//   return new TextEncoder().encode(`Echo: <${message}>`)
// }

// function getPeerProtocol(peer: PeerId, protocol: string) {
//   return `${peer.toString()}${protocol}`
// }

// function createFakeComponents(peerId: PeerId, network: EventEmitter): Components {
//   return {
//     getPeerId() {
//       return peerId
//     },
//     getRegistrar() {
//       return {
//         handle(protocol: string, handler: (conn: StreamHandler) => void) {
//           network.on(getPeerProtocol(peerId, protocol), handler)
//         }
//       } as Components['registrar']
//     },
//     getUpgrader() {
//       return {
//         upgradeInbound: (async (conn: RelayConnection) => {
//           const shaker = handshake(conn)

//           const message = new TextDecoder().decode(((await shaker.read()) as Uint8Array).slice())

//           shaker.write(msgToEchoedMessage(message))

//           shaker.rest()
//         }) as any,
//         upgradeOutbound: (conn: any) => conn
//       }
//     },
//     getPeerStore() {
//       return {
//         addressBook: {
//           get: async (peer: PeerId): Promise<Address[]> => {
//             return [
//               {
//                 multiaddr: new Multiaddr(`/ip4/127.0.0.1/tcp/1/p2p/${peer.toString()}`),
//                 isCertified: true
//               }
//             ]
//           }
//         }
//       }
//     },
//     getConnectionManager() {
//       return {
//         getConnections(_peerId: PeerId) {
//           return []
//         },
//         dialer: {} as any
//       } as Components['connectionManager']
//     }
//   } as Components
// }

// function getPeer(peerId: PeerId, network: EventEmitter, testingOptions?: HoprConnectTestingOptions) {
//   async function dialDirectly(ma: Multiaddr): Promise<Connection> {
//     const peerId = peerIdFromString(ma.getPeerId() as string)

//     return {
//       remotePeer: peerId,
//       newStream: async (protocol: string) => {
//         const AtoB = pair<StreamType>()
//         const BtoA = pair<StreamType>()

//         network.emit(getPeerProtocol(peerId, protocol), {
//           stream: {
//             sink: AtoB.sink,
//             source: BtoA.source
//           },
//           connection: {
//             remotePeer: peerId
//           }
//         })

//         return {
//           protocol,
//           stream: {
//             sink: BtoA.sink,
//             source: AtoB.source
//           }
//         }
//       }
//     } as any
//   }

//   const relay = new Relay(
//     dialDirectly,
//     (multiaddrs: Multiaddr[]) => multiaddrs,
//     { environment: `testingEnvironment` },
//     testingOptions ?? { __noWebRTCUpgrade: true }
//   )

//   relay.init(createFakeComponents(peerId, network))
//   relay.initConnect({
//     getWebRTCUpgrader() {
//       const webRTCInstance = new EventEmitter()
//       return {
//         upgradeOutbound() {
//           return webRTCInstance
//         },
//         upgradeInbound() {
//           return webRTCInstance
//         }
//       }
//     }
//   } as ConnectComponents)

//   relay.start()
//   relay.afterStart()

//   return relay
// }

// describe('test relay', function () {
//   it('connect to a relay, close the connection and reconnect', async function () {
//     const network = new EventEmitter()

//     const Alice = getPeer(initiator, network)
//     const Bob = getPeer(relay, network)
//     const Charly = getPeer(counterparty, network)

//     for (let i = 0; i < 5; i++) {
//       const conn = await Alice.connect(Bob.getComponents().getPeerId(), Charly.getComponents().getPeerId())

//       assert(conn != undefined, `Should be able to connect`)
//       const shaker = handshake(conn as any)

//       const msg = '<Hello>, that should be sent and echoed through relayed connection'
//       shaker.write(new TextEncoder().encode(msg))

//       assert(u8aEquals(((await shaker.read()) as Uint8Array).slice(), msgToEchoedMessage(msg)))

//       shaker.rest()

//       await conn.close()

//       // Let I/O happen
//       await new Promise((resolve) => setTimeout(resolve))
//     }

//     Alice.stop()
//     Bob.stop()
//     Charly.stop()

//     network.removeAllListeners()
//   })

//   it('connect to a relay and reconnect', async function () {
//     const network = new EventEmitter()

//     const Alice = getPeer(initiator, network)
//     const Bob = getPeer(relay, network)
//     const Charly = getPeer(counterparty, network)

//     for (let i = 0; i < 3; i++) {
//       const conn = await Alice.connect(Bob.getComponents().getPeerId(), Charly.getComponents().getPeerId())

//       assert(conn != undefined, `Should be able to connect`)
//       const shaker = handshake(conn as any)

//       const msg = '<Hello>, that should be sent and echoed through relayed connection'
//       shaker.write(new TextEncoder().encode(msg))

//       assert(u8aEquals(((await shaker.read()) as Uint8Array).slice(), msgToEchoedMessage(msg)))

//       shaker.rest()

//       // Let I/O happen
//       await new Promise((resolve) => setTimeout(resolve))
//     }

//     Alice.stop()
//     Bob.stop()
//     Charly.stop()

//     network.removeAllListeners()
//   })
// })
