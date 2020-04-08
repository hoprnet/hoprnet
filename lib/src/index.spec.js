// import PeerId from 'peer-id'
// import { Packet } from './messages/packet'
// import HoprPolkadot, { Utils, Types } from '@hoprnet/hopr-core-polkadot'
// import { Interactions } from './interactions'
// import { Channel } from '@hoprnet/hopr-core-polkadot/src/channel'
// import Hopr from '.'
// import Debug from 'debug'
// import LevelUp from 'levelup'
// import Memdown from 'memdown'
// describe(`create, transform packet with message, header, challenge and payments`, function() {
//   it('should create a packet', async function() {
//     const peers = await Promise.all(
//       [undefined, undefined, undefined, undefined].map(_ =>
//         PeerId.create({
//           keyType: 'secp256k1'
//         })
//       )
//     )
//     const node = ({
//       log: Debug('test'),
//       peerInfo: {
//         id: peers[0]
//       },
//       paymentChannels: await HoprPolkadot.create(
//         new LevelUp(Memdown()),
//         {
//           publicKey: Uint8Array.from(peers[0].pubKey.marshal()),
//           privateKey: Uint8Array.from(peers[0].privKey.marshal())
//         },
//         `ws://localhost:9944`
//       )
//     } as unknown) as Hopr<HoprPolkadot>
//     console.log(`connected`)
//     const testMessage = new TextEncoder().encode('test')
//     const packet = await Packet.create(node, testMessage, peers.slice(1))
//     console.log(packet)
//   })
// })
