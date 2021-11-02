#!/usr/bin/env -S yarn --silent ts-node
// Used to get a node's spoken protocols and to test connectivity
// Example usage: `./scripts/get-supported-protocols-from-peer.ts --addr /ip4/34.65.6.139/tcp/9091/p2p/16Uiu2HAm5Ym8minpwct7aZ9dYYnpbjfsfr8wa6o7GbRFcmXLcmFW`

import pipe from 'it-pipe'
import chalk from 'chalk'
import yargs from 'yargs/yargs'

import { Multiaddr } from 'multiaddr'
import type PeerId from 'peer-id'
import { NOISE } from '@chainsafe/libp2p-noise'
import Upgrader from 'libp2p/src/upgrader'
import MPLEX from 'libp2p-mplex'
const Multistream = require('multistream-select')

const { HoprConnect } = require('@hoprnet/hopr-connect')
import { privKeyToPeerId, stringToU8a } from '@hoprnet/hopr-utils'

const id = '0x964e55c734330e9393a32aef61e6b75f3b526fb64df0ac55a6076045918657e1'

// @ts-ignore
class ReducedUpgrader extends Upgrader {
  /**
   * Overwriting a convenience method in libp2p's Upgrader class to get the spoken
   * protocols before using the connection
   *
   * @override
   * @param {object} options
   * @param {MultiaddrConnection} options.maConn - The transport layer connection
   * @param {MuxedStream | MultiaddrConnection} options.upgradedConn - A duplex connection returned from multiplexer and/or crypto selection
   * @param {MuxerFactory} [options.Muxer] - The muxer to be used for muxing
   * @param {PeerId} options.remotePeer - The peer the connection is with
   */
  // @ts-ignore
  private _createConnection({ maConn, upgradedConn, Muxer, remotePeer }): {
    getProtocols: () => Promise<string[]>
    remotePeer: PeerId
    close: () => Promise<void>
  } {
    let muxer = new Muxer()

    let getProtocols = async () => {
      // log('%s: starting new stream on %s', direction, protocols)
      const muxedStream = muxer.newStream()
      const mss = new Multistream.Dialer(muxedStream)

      return await mss.ls()
    }

    // Pipe all data through the muxer
    pipe(upgradedConn, muxer, upgradedConn) //.catch(log.error)

    maConn.timeline.upgraded = Date.now()

    const close = async () => {
      await maConn.close()
      // Ensure remaining streams are aborted
      if (muxer) {
        muxer.streams.map((stream) => stream.abort())
      }
    }

    return {
      getProtocols,
      close,
      remotePeer
    }
  }
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

  const self = privKeyToPeerId(stringToU8a(id, 32))

  // @ts-ignore
  const upgrader = new ReducedUpgrader({
    localPeer: self
  })

  // As used by other HOPR nodes
  upgrader.cryptos.set(NOISE.protocol, NOISE)
  upgrader.muxers.set(MPLEX.multicodec, MPLEX)

  // Use minimal configuration for hopr-connect
  const Transport = new HoprConnect({
    upgrader,
    libp2p: {
      peerId: self,
      handle: () => {}
    }
  })

  let _conn: ReturnType<ReducedUpgrader['_createConnection']>

  // Try to dial node and fail if there was an error
  try {
    _conn = await Transport.dial(ma)
  } catch (err) {
    console.log(err)
    return
  }

  const protocols = await _conn.getProtocols()

  console.log(
    `Node identified as ${chalk.blue(_conn.remotePeer.toB58String())}, supported protocol${
      protocols.length == 1 ? '' : 's'
    }:\n  ${protocols.map((str) => chalk.green(str)).join('\n  ')}`
  )

  await _conn.close()
}

main()
