// import { PeerId } from '@libp2p/interface-peer-id'
// import { Multiaddr } from '@multiformats/multiaddr'
// import type { PeerStore } from '@libp2p/interface-peer-store'
// import AddressManager from 'libp2p/src/address-manager/index.js'
// import { MemoryDatastore } from 'datastore-core/memory'

// import { debug } from '@hoprnet/hopr-utils'

// import type { Libp2p } from 'libp2p'

// function createLibp2pMock(peerId: PeerId): Libp2p {
//   const libp2pLogger = debug(`hopr:mocks:libp2p`)

//   const libp2p = {} as unknown as Libp2p

//   libp2p.peerId = peerId

//   libp2p._options = Object.assign({}, libp2p._options, {
//     addresses: {
//       announceFilter: () => [new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${peerId.toString()}`)]
//     }
//   })
//   libp2p.start = () => {
//     libp2pLogger(`Libp2p start method called`)
//     return Promise.resolve()
//   }
//   libp2p.stop = () => {
//     libp2pLogger(`Libp2p stop method called`)
//     return Promise.resolve()
//   }
//   libp2p.handle = async () => {
//     libp2pLogger(`Libp2 handle method called`)
//   }
//   libp2p.hangUp = () => {
//     libp2pLogger(`Libp2 hangUp method called`)
//     return Promise.resolve()
//   }
//   libp2p.connectionManager = {} as unknown as Libp2p['connectionManager']
//   libp2p.connectionManager.addEventListener = (event: string) => {
//     libp2pLogger(`Connection manager event handler called with event "${event}"`)
//     return libp2p.connectionManager
//   }
//   const datastore = new MemoryDatastore()
//   const addressFilter = async () => Promise.resolve(true)
//   libp2p.peerStore = new PeerStore({ peerId, datastore, addressFilter })
//   libp2p.addressManager = new AddressManager(peerId, {
//     announce: [new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${peerId.toString()}`).toString()]
//   })

//   libp2p.upgrader = {} as any

//   return libp2p
// }

// export { createLibp2pMock }
