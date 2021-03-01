import libp2p from 'libp2p'
import type { Handler, Stream } from 'libp2p'
import { durations } from '@hoprnet/hopr-utils'

import { NOISE } from 'libp2p-noise'

const MPLEX = require('libp2p-mplex')

import HoprConnect from '../src'
import Multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import { Alice, Bob, Charly } from './identities'
import pipe from 'it-pipe'

const TEST_PROTOCOL = '/hopr-connect/test/0.0.1'

async function main() {
  const RELAY_ADDRESS = Multiaddr(`/ip4/127.0.0.1/tcp/9092/p2p/${await PeerId.createFromPrivKey(Charly)}`)

  let peerId: PeerId
  let port: number
  switch (process.argv[2]) {
    case '0':
      peerId = await PeerId.createFromPrivKey(Alice)
      port = 9090
      break
    case '1':
      peerId = await PeerId.createFromPrivKey(Bob)
      port = 9091
      break
    default:
      console.log(`Invalid CLI options. Either run with '0' or '1'. Got ${process.argv[2]}`)
      process.exit()
  }

  const node = await libp2p.create({
    peerId,
    addresses: {
      listen: [Multiaddr(`/ip4/0.0.0.0/tcp/${port}/p2p/${peerId.toB58String()}`)]
    },
    modules: {
      transport: [HoprConnect],
      streamMuxer: [MPLEX],
      connEncryption: [NOISE]
    },
    config: {
      transport: {
        HoprConnect: {
          bootstrapServers: [RELAY_ADDRESS],
          // simulates a NAT
          // DO NOT use this in production
          __noDirectConnections: true,
          __noWebRTCUpgrade: false
        }
      },
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

  node.handle(TEST_PROTOCOL, (struct: Handler) => {
    pipe(
      struct.stream.source,
      (source: Stream['source']) => {
        return (async function* () {
          for await (const msg of source) {
            const decoded = new TextDecoder().decode(msg.slice())

            console.log(`Received message <${decoded}>`)

            yield new TextEncoder().encode(`Echoing <${decoded}>`)
          }
        })()
      },
      struct.stream.sink
    )
  })

  await node.dial(RELAY_ADDRESS)

  console.log(`giving counterparty time to start`)
  await new Promise((resolve) => setTimeout(resolve, durations.seconds(8)))
  console.log(`end Timeout`)

  //@ts-ignore
  let conn: Handler

  switch (process.argv[2]) {
    case '0':
      try {
        conn = await node.dialProtocol(
          Multiaddr(
            `/p2p/${await PeerId.createFromPrivKey(Charly)}/p2p-circuit/p2p/${await PeerId.createFromPrivKey(Bob)}`
          ),
          TEST_PROTOCOL
        )
      } catch (err) {
        console.log(err)
        return
      }

      await pipe(
        // prettier-ignore
        // async function * () {
        //   let i = 0
        //   while(true) {
        //     yield new TextEncoder().encode(`test ${i}`)

        //     await new Promise(resolve => setTimeout(resolve, 100))
        //     i++
        //   }
        // }(),
        [new TextEncoder().encode(`test`)],
        conn.stream,
        async (source: Stream['source']) => {
          for await (const msg of source) {
            const decoded = new TextDecoder().decode(msg.slice())

            console.log(`Received <${decoded}>`)
          }
        }
      )

      break
    // case '1':
    //   conn = await node.dialProtocol(
    //     Multiaddr(`/ip4/127.0.0.1/tcp/9090/p2p/${await PeerId.createFromPrivKey(Alice)}`),
    //     TEST_PROTOCOL
    //   )

    //   break
    // default:
    //   console.log(`Invalid CLI options. Either run with '0' or '1'. Got ${process.argv[2]}`)
    //   process.exit()
  }

  console.log('running')
}

main()
