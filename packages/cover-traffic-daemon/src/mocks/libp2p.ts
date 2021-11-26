import LibP2P from 'libp2p'
import ConnectionManager from 'libp2p/src/connection-manager'
import PeerStore from 'libp2p/src/peer-store'
import AddressManager from 'libp2p/src/address-manager'
import { debug } from '@hoprnet/hopr-utils'
import { NAMESPACE, mockPeerId, sampleMultiaddrs, samplePeerId } from './constants'

const libp2pLogger = debug(`${NAMESPACE}:libp2p`)
let libp2p: LibP2P

libp2p = {} as unknown as LibP2P
libp2p._options = Object.assign({}, libp2p._options, {
  addresses: {
    announceFilter: () => [sampleMultiaddrs]
  }
})
libp2p.start = () => {
  libp2pLogger(`Libp2p start method called`)
  return Promise.resolve();
}
libp2p.stop = () => {
  libp2pLogger(`Libp2p stop method called`)
  return Promise.resolve();
}
libp2p.handle = () => {
  libp2pLogger(`Libp2 handle method called`)
}
libp2p.hangUp = () => {
  libp2pLogger(`Libp2 hangUp method called`)
  return Promise.resolve();
}
libp2p.connectionManager = {} as unknown as ConnectionManager
libp2p.connectionManager.on = (event: string) => {
  libp2pLogger(`Connection manager event handler called with event "${event}"`)
  return libp2p.connectionManager;
}
libp2p.peerStore = new PeerStore({ peerId: samplePeerId })
libp2p.addressManager = new AddressManager(mockPeerId, { announce: [sampleMultiaddrs.toString()] })


const libp2pMock = libp2p
export { libp2pMock }
