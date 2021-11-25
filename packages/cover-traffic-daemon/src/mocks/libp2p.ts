import LibP2P from 'libp2p'
import ConnectionManager from 'libp2p/src/connection-manager'
import PeerStore from 'libp2p/src/peer-store'
import AddressManager from 'libp2p/src/address-manager'
import { debug } from "@hoprnet/hopr-utils"
import { NAMESPACE, mockPeerId, sampleMultiaddrs, samplePeerId } from "./constants"
import sinon from 'sinon';

const libp2pLogger = debug(`${NAMESPACE}:libp2p`)
let libp2p: LibP2P

libp2p = sinon.createStubInstance(LibP2P)
libp2p._options = Object.assign({}, libp2p._options, {
  addresses: {
    announceFilter: () => [sampleMultiaddrs]
  }
})
libp2p.connectionManager = sinon.createStubInstance(ConnectionManager)
libp2p.connectionManager.on = sinon.fake((event: string) => {
  libp2pLogger(`Connection manager event handler called with event "${event}"`)
})
libp2p.peerStore = new PeerStore({ peerId: samplePeerId })
libp2p.addressManager = new AddressManager(mockPeerId, { announce: [sampleMultiaddrs.toString()] })

function stubLibp2p() {
  sinon.stub(LibP2P, 'create').callsFake(() => {
    libp2pLogger('libp2p stub started')
    return Promise.resolve(libp2p)
  })
}

export { stubLibp2p }