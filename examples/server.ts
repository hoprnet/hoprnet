import libp2p from 'libp2p'

import { NOISE } from 'libp2p-noise'
const MPLEX = require('libp2p-mplex')

import HoprConnect from '../src'
import { Charly } from './identities'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

async function main() {
  const node = await libp2p.create({
    peerId: await PeerId.createFromPrivKey(Charly),
    addresses: {
      listen: [Multiaddr(`/ip4/0.0.0.0/tcp/9092/p2p/${(await PeerId.createFromPrivKey(Charly)).toB58String()}`)]
    },
    modules: {
      transport: [HoprConnect],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE]
    },
    dialer: {
      // Temporary fix
      addressSorter: (ma: Multiaddr) => ma
    }
  })

  await node.start()

  console.log(`running`)
}

main()
