import libp2p from 'libp2p'

import { NOISE } from 'libp2p-noise'
const MPLEX = require('libp2p-mplex')

import { HoprConnect } from '../src'
import { getIdentity } from './identities'
import PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'
import yargs from 'yargs/yargs'

async function main() {
  const argv = yargs(process.argv.slice(2))
    .option('serverPort', {
      describe: 'server port name',
      type: 'number',
      demandOption: true
    })
    .option('serverIdentityName', {
      describe: 'server identity name',
      choices: ['alice', 'bob', 'charly', 'dave', 'ed'],
      demandOption: true
    })
    .parseSync()

  const serverPeerId = await PeerId.createFromPrivKey(getIdentity(argv.serverIdentityName))

  const node = await libp2p.create({
    peerId: serverPeerId,
    addresses: {
      listen: [new Multiaddr(`/ip4/0.0.0.0/tcp/${argv.serverPort}/p2p/${serverPeerId.toB58String()}`)]
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

  console.log(`running server ${argv.serverIdentityName} on port ${argv.serverPort}`)
}

main()
