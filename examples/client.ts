import libp2p from 'libp2p'
import type { Handler, Stream } from 'libp2p'
import { durations } from '@hoprnet/hopr-utils'

import { NOISE } from 'libp2p-noise'

const MPLEX = require('libp2p-mplex')

import { HoprConnect } from '../src'
import { Multiaddr } from 'multiaddr'
import PeerId from 'peer-id'
import { getIdentity } from './identities'
import pipe from 'it-pipe'

const TEST_PROTOCOL = '/hopr-connect/test/0.0.1'

async function main() {
  const clientPort = process.argv[3]
  const clientIdentityName = process.argv[4]
  const relayPort = process.argv[5]
  const relayPeerId = await PeerId.createFromPrivKey(getIdentity(process.argv[6]))
  let counterPartyPeerId: PeerId | null = null
  if (process.argv[7]) {
    counterPartyPeerId = await PeerId.createFromPrivKey(getIdentity(process.argv[7]))
  }

  const RELAY_ADDRESS = new Multiaddr(`/ip4/127.0.0.1/tcp/${relayPort}/p2p/${relayPeerId.toB58String()}`)

  const clientPeerId = await PeerId.createFromPrivKey(getIdentity(clientIdentityName))

  const node = await libp2p.create({
    peerId: clientPeerId,
    addresses: {
      listen: [new Multiaddr(`/ip4/0.0.0.0/tcp/${clientPort}/p2p/${clientPeerId.toB58String()}`)]
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

  console.log(`running client ${clientIdentityName} on port ${clientPort}`)

  console.log(`giving counterparty time to start`)
  await new Promise((resolve) => setTimeout(resolve, durations.seconds(8)))
  console.log(`end Timeout`)

  //@ts-ignore
  let conn: Handler

  if (counterPartyPeerId)
    try {
      conn = await node.dialProtocol(
        new Multiaddr(`/p2p/${relayPeerId}/p2p-circuit/p2p/${counterPartyPeerId.toB58String()}`),
        TEST_PROTOCOL
      )
      await pipe([new TextEncoder().encode(`test`)], conn.stream, async (source: Stream['source']) => {
        for await (const msg of source) {
          const decoded = new TextDecoder().decode(msg.slice())

          console.log(`Received <${decoded}>`)
        }
      })
    } catch (err) {
      console.log(err)
      return
    }
}

main()
