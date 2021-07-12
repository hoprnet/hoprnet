import libp2p from 'libp2p'

import { NOISE } from 'libp2p-noise'
const MPLEX = require('libp2p-mplex')

import { HoprConnect } from '../src'
import { getIdentity } from './identities'
import PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'

async function main() {
  const serverPort = process.argv[2]
  const serverIdentityName = process.argv[3]
  const serverPeerId = await PeerId.createFromPrivKey(getIdentity(serverIdentityName))

  const node = await libp2p.create({
    peerId: serverPeerId,
    addresses: {
      listen: [new Multiaddr(`/ip4/0.0.0.0/tcp/${serverPort}/p2p/${serverPeerId.toB58String()}`)]
    },
    modules: {
      transport: [HoprConnect],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE]
    },
    config: {
      peerDiscovery: {
        autoDial: false
      }
    },
    dialer: {
      // Temporary fix
      addressSorter: (ma: Multiaddr) => ma
    }
  })

  await node.start()

  console.log(`running server ${serverIdentityName} on port ${serverPort}`)
}

main()
