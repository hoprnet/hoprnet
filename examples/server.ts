import libp2p from 'libp2p'

const SECIO = require('libp2p-secio')
const MPLEX = require('libp2p-mplex')

import HoprConnect from '../src'
import { Charly } from './identities'
import PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

async function main() {
  const node = await libp2p.create({
    peerId: await PeerId.createFromPrivKey(Charly),
    addresses: {
      listen: [Multiaddr(`/ip4/0.0.0.0/tcp/9092`)]
    },
    modules: {
      transport: [HoprConnect],
      streamMuxer: [MPLEX],
      connEncryption: [SECIO]
    }
  })

  await node.start()

  console.log(`running`)
}

main()
