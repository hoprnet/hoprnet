#!/usr/bin/env -S yarn --silent ts-node
// Early beta - to be tested soon
// Used to get a node's spoken protocols and to test connectivity
// Example usage: `./scripts/get-supported-protocols-from-peer.ts --addr /ip4/34.65.6.139/tcp/9091/p2p/16Uiu2HAm5Ym8minpwct7aZ9dYYnpbjfsfr8wa6o7GbRFcmXLcmFW`

import chalk from 'chalk'
import yargs from 'yargs/yargs'
import debug from 'debug'

import { Multiaddr } from '@multiformats/multiaddr'
import type { PeerId } from '@libp2p/interface-peer-id'
import { Noise } from '@chainsafe/libp2p-noise'
import type { StreamMuxerFactory } from '@libp2p/interfaces/stream-muxer'
import { Mplex } from '@libp2p/mplex'
import { createLibp2p, type Libp2p as Libp2pType } from 'libp2p'
import type { Components } from '@libp2p/interfaces/components'
import { EventEmitter } from 'events'
import { Dialer } from '@libp2p/multistream-select'
import type { MultiaddrConnection } from '@libp2p/interface-connection'
import { pipe } from 'it-pipe'
import { Duplex } from 'it-stream-types'

import { HoprConnect } from '@hoprnet/hopr-connect'
import { privKeyToPeerId } from '@hoprnet/hopr-utils'

const log = debug('hopr:ls-protocols')

interface Libp2p extends Libp2pType {
  components: Components
}

interface MyCreateConnection {
  // to speak Multistream-`ls` protocol
  dialer: Dialer
  // to close connection
  maConn: MultiaddrConnection
  // to identify remotePeer
  remotePeer: PeerId
}

/**
 * Creates a libp2p instance with a similar configuration as used
 * in `core` but without any `hopr`-specific functionalities
 */
async function getLibp2pInstance(): Promise<Libp2p> {
  const peerId = privKeyToPeerId('0x964e55c734330e9393a32aef61e6b75f3b526fb64df0ac55a6076045918657e1')

  const libp2p = (await createLibp2p({
    peerId,
    addresses: { listen: [`/ip4/127.0.0.1/tcp/0/${peerId.toString()}`] },
    transports: [
      // @ts-ignore libp2p interface type clash
      new HoprConnect({
        config: {
          publicNodes: new EventEmitter(),
          allowLocalConnections: true,
          allowPrivateConnections: true,
          // Amount of nodes for which we are willing to act as a relay
          maxRelayedConnections: 50_000
        }
      })
    ],
    streamMuxers: [new Mplex()],
    connectionEncryption: [new Noise()],
    connectionManager: {
      autoDial: true,
      // Use custom sorting to prevent from problems with libp2p
      // and HOPR's relay addresses
      addressSorter: () => 0,
      // Don't try to dial a peer using multiple addresses in parallel
      maxDialsPerPeer: 1,
      // If we are a public node, assume that our system is able to handle
      // more connections
      maxParallelDials: 50,
      // default timeout of 30s appears to be too long
      dialTimeout: 10e3
    },
    relay: {
      // Conflicts with HoprConnect's own mechanism
      enabled: false
    },
    nat: {
      // Conflicts with HoprConnect's own mechanism
      enabled: false
    }
  })) as Libp2p

  await libp2p.start()

  // Total hack
  // @ts-ignore
  Object.assign(libp2p.components.upgrader, {
    _createConnection: (opts: {
      cryptoProtocol: string
      direction: 'inbound' | 'outbound'
      maConn: MultiaddrConnection
      upgradedConn: Duplex<Uint8Array>
      remotePeer: PeerId
      muxerFactory?: StreamMuxerFactory
    }): MyCreateConnection => {
      const muxer = opts.muxerFactory.createStreamMuxer(libp2p.components)

      // Pipe all data through the muxer
      pipe(opts.upgradedConn, muxer, opts.upgradedConn).catch(log)

      return { dialer: new Dialer(muxer.newStream()), maConn: opts.maConn, remotePeer: opts.remotePeer }
    }
  })

  return libp2p
}

async function main() {
  const argv = yargs(process.argv.slice(2))
    .option('addr', {
      describe: 'example: --addr /ip4/34.65.42.178/tcp/9091/p2p/16Uiu2HAkyQGg2LLqwbDbuiHZSVtB3q5xmhpq7URirCEuJ4CXjZTh',
      demandOption: true,
      type: 'string'
    })
    .parseSync()

  let ma: Multiaddr
  try {
    ma = new Multiaddr(argv.addr)
  } catch (err) {
    console.log(`Error while decoding Multiaddr: `, err)
    return
  }

  const libp2p = await getLibp2pInstance()

  // @ts-ignore non-public api
  const conn = (await libp2p.components.getConnectionManager().dialer.dial(ma)) as MyCreateConnection

  const protocols = await conn.dialer.ls()

  console.log(
    `Node identified as ${chalk.blue(conn.remotePeer.toString())}, supported protocol${
      protocols.length == 1 ? '' : 's'
    }:\n  ${protocols.map((str) => chalk.green(str)).join('\n  ')}`
  )

  try {
    conn.maConn.close()
  } catch (err) {
    log(`Error while closing connection`, err)
  }
}

main()
