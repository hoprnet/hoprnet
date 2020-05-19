import assert from 'assert'

// @ts-ignore
import libp2p = require('libp2p')

// @ts-ignore
import MPLEX = require('libp2p-mplex')
// @ts-ignore
import KadDHT = require('libp2p-kad-dht')
// @ts-ignore
import SECIO = require('libp2p-secio')

import PeerId from 'peer-id'

import { MultiaddrConnection, Handler } from './types'

import TCP from '.'
import Multiaddr from 'multiaddr'
import PeerInfo from 'peer-info'
import pipe from 'it-pipe'

import { u8aEquals } from '@hoprnet/hopr-utils'

import chalk from 'chalk'

// @ts-ignore
import miniStun = require('ministun')

const TEST_PROTOCOL = `/test/0.0.1`

describe('should create a socket and connect to it', function () {
  const upgrader = {
    upgradeOutbound: async (maConn: MultiaddrConnection) => maConn,
    upgradeInbound: async (maConn: MultiaddrConnection) => maConn,
  }

  async function startStun() {
    const server = new miniStun({
      udp4: true,
      upd6: true,
      port: 3478,
      log: null,
      err: null,
      sw: true,
    })
    
    await server.start()

    return server
  }

  async function generateNode(
    options: {
      id: number
      ipv4?: boolean
      ipv6?: boolean
    },
    bootstrap?: PeerInfo
  ): Promise<libp2p> {
    const peerInfo = new PeerInfo(await PeerId.create({ keyType: 'secp256k1' }))

    if (options.ipv4) {
      peerInfo.multiaddrs.add(
        Multiaddr(`/ip4/127.0.0.1/tcp/${9090 + 2 * options.id}`).encapsulate(`/p2p/${peerInfo.id.toB58String()}`)
      )
    }

    if (options.ipv6) {
      peerInfo.multiaddrs.add(
        Multiaddr(`/ip6/::1/tcp/${9090 + 2 * options.id + 1}`).encapsulate(`/p2p/${peerInfo.id.toB58String()}`)
      )
    }

    const node = new libp2p({
      peerInfo,
      modules: {
        transport: [TCP],
        streamMuxer: [MPLEX],
        connEncryption: [SECIO],
        dht: KadDHT,
      },
      config: {
        transport: {
          TCP: {
            bootstrapServers: [bootstrap],
          },
        },
        dht: {
          enabled: false,
        },
        relay: {
          enabled: false,
        },
        peerDiscovery: {
          autoDial: false,
        },
      },
    })

    node.handle(TEST_PROTOCOL, (handler: Handler) => {
      pipe(
        /* prettier-ignore */
        handler.stream,
        (source: AsyncIterable<Uint8Array>) => {
          return (async function* () {
            for await (const msg of source) {
              yield msg
            }
          })()
        },
        handler.stream
      )
    })

    await node.start()

    return node
  }

  it('should set up a socket', async function () {
    // const server = await startStun()

    const relay = await generateNode({ id: 2, ipv4: true, ipv6: true })

    const [sender, counterparty] = await Promise.all([
      generateNode({ id: 0, ipv4: true }, relay.peerInfo),
      generateNode({ id: 1, ipv6: true }, relay.peerInfo),
    ])

    console.log(`Sender       ${chalk.yellow(sender.peerInfo.id.toB58String())}`)
    console.log(`Counterparty ${chalk.yellow(counterparty.peerInfo.id.toB58String())}\n`)

    console.log(`Relay        ${chalk.yellow(relay.peerInfo.id.toB58String())}\n`)



    connectionHelper([sender, relay])
    connectionHelper([relay, counterparty])

    // const conn1 = await sender.dial(counterparty.peerInfo)

    // sender.peerStore.remove(counterparty.peerInfo.id)
    // await sender.hangUp(counterparty.peerInfo)

    const INVALID_PORT = 8758
    const conn2 = await sender.dialProtocol(
      Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}/p2p/${counterparty.peerInfo.id.toB58String()}`),
      TEST_PROTOCOL
    )

    const testMessage = new TextEncoder().encode('12356')
    await pipe(
      /* prettier-ignore */
      [testMessage],
      conn2.stream,
      async (source: AsyncIterable<Uint8Array>) => {
        for await (const msg of source) {
          assert(u8aEquals(msg.slice(), testMessage), 'sent message and received message must be identical')
          console.log(`message received`)
          return
        }
      }
    )
  })
})

/**
 * Informs each node about the others existence.
 * @param nodes Hopr nodes
 */
function connectionHelper(nodes: libp2p[]) {
  for (let i = 0; i < nodes.length; i++) {
    for (let j = i + 1; j < nodes.length; j++) {
      nodes[i].peerStore.put(nodes[j].peerInfo)
      nodes[j].peerStore.put(nodes[i].peerInfo)
    }
  }
}
