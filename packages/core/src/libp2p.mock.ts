import LibP2P from 'libp2p'
import ConnectionManager from 'libp2p/src/connection-manager'
import PeerStore from 'libp2p/src/peer-store'
import AddressManager from 'libp2p/src/address-manager'
import { debug, privKeyToPeerId } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'

export const privateKey = '0xcb1e5d91d46eb54a477a7eefec9c87a1575e3e5384d38f990f19c09aa8ddd332'
export const mockPeerId = privKeyToPeerId(privateKey)
export const samplePeerId = PeerId.createFromB58String('16Uiu2HAmThyWP5YWutPmYk9yUZ48ryWyZ7Cf6pMTQduvHUS9sGE7')
export const sampleMultiaddrs = new Multiaddr(`/ip4/127.0.0.1/tcp/124/p2p/${samplePeerId.toB58String()}`)

const libp2pLogger = debug(`hopr:mocks:libp2p`)
let libp2p: LibP2P

libp2p = {} as unknown as LibP2P
libp2p._options = Object.assign({}, libp2p._options, {
  addresses: {
    announceFilter: () => [sampleMultiaddrs]
  }
})
libp2p.start = () => {
  libp2pLogger(`Libp2p start method called`)
  return Promise.resolve()
}
libp2p.stop = () => {
  libp2pLogger(`Libp2p stop method called`)
  return Promise.resolve()
}
libp2p.handle = () => {
  libp2pLogger(`Libp2 handle method called`)
}
libp2p.hangUp = () => {
  libp2pLogger(`Libp2 hangUp method called`)
  return Promise.resolve()
}
libp2p.connectionManager = {} as unknown as ConnectionManager
libp2p.connectionManager.on = (event: string) => {
  libp2pLogger(`Connection manager event handler called with event "${event}"`)
  return libp2p.connectionManager
}
libp2p.peerStore = new PeerStore({ peerId: samplePeerId })
libp2p.addressManager = new AddressManager(mockPeerId, { announce: [sampleMultiaddrs.toString()] })

const libp2pMock = libp2p
export { libp2pMock }
