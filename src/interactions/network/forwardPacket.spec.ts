// import PeerInfo from 'peer-info'
// import PeerId from 'peer-id'

// // @ts-ignore
// import libp2p = require('libp2p')
// // @ts-ignore
// import TCP = require('libp2p-tcp')
// // @ts-ignore
// import MPLEX = require('libp2p-mplex')
// // @ts-ignore
// import SECIO = require('libp2p-secio')

// import Debug from 'debug'
// import chalk from 'chalk'

// import Hopr from '../..'
// import HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'
// import { ForwardPacketInteraction } from './forwardPacket'

// import { randomBytes } from 'crypto'
// import assert from 'assert'
// import Multiaddr from 'multiaddr'

// import pipe from 'it-pipe'
// import { u8aEquals } from '@hoprnet/hopr-utils'

// describe('check packet forward mechanism', function () {
//   async function generateNode(): Promise<Hopr<HoprCoreConnector>> {
//     const node = (await libp2p.create({
//       peerInfo: await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' })),
//       modules: {
//         transport: [TCP],
//         streamMuxer: [MPLEX],
//         connEncryption: [SECIO],
//       },
//     })) as Hopr<HoprCoreConnector>

//     node.peerInfo.multiaddrs.add(Multiaddr('/ip4/0.0.0.0/tcp/0'))

//     await node.start()

//     node.peerRouting.findPeer = (_: PeerId): Promise<never> => {
//       return Promise.reject(Error('not implemented'))
//     }

//     node.interactions = {
//       network: {
//         forwardPacket: new ForwardPacketInteraction(node),
//       },
//     } as Hopr<HoprCoreConnector>['interactions']

//     node.log = Debug(`${chalk.blue(node.peerInfo.id.toB58String())}: `)

//     return (node as unknown) as Hopr<HoprCoreConnector>
//   }

//   it('should forward a packet', async function () {
//     const [Alice, Relay, Bob] = await Promise.all([generateNode(), generateNode(), generateNode()])

//     connectionHelper(Alice, Relay, Bob)

//     const conn = await Alice.interactions.network.forwardPacket.interact(
//       Bob.peerInfo.id,
//       Relay.peerInfo
//     )

//     const otherSide = await Bob.interactions.network.forwardPacket.interact(
//       Alice.peerInfo.id,
//       Relay.peerInfo
//     )

//     const firstMessage = randomBytes(31)
//     const secondMessage = randomBytes(41)

//     let firstMessageReceived = false
//     let secondMessageReceived = true

//     await pipe(
//       /* prettier-ignore */
//       otherSide,
//       (source: any) => {
//         return (async function* () {
//           for await (let msg of source) {
//             assert(u8aEquals(msg.slice(), firstMessage))
//             firstMessageReceived = true
//             yield secondMessage
//           }
//         })()
//       },
//       otherSide
//     )

//     await pipe(
//       /* prettier-ignore */
//       [firstMessage],
//       conn,
//       async (source: any) => {
//         for await (let msg of source) {
//           assert(u8aEquals(msg.slice(), secondMessage))
//           secondMessageReceived = true

//           // ends the stream
//           conn.source.end()
//         }
//       }
//     )

//     assert(firstMessageReceived, `Message exchange must have happened`)
//     assert(secondMessageReceived, `Message exchange must have happened`)

//     await Promise.all([Alice.stop(), Bob.stop(), Relay.stop()])
//   })
// })

// /**
//  * Informs each node about the others existence.
//  * @param nodes Hopr nodes
//  */
// function connectionHelper<Chain extends HoprCoreConnector>(...nodes: Hopr<Chain>[]): void {
//   for (let i = 0; i < nodes.length; i++) {
//     for (let j = i + 1; j < nodes.length; j++) {
//       nodes[i].peerStore.put(nodes[j].peerInfo)
//       nodes[j].peerStore.put(nodes[i].peerInfo)
//     }
//   }
// }
