const MPLEX = require('libp2p-mplex')
import { NOISE } from 'libp2p-noise'
const Multistream = require('multistream-select')
const { HoprConnect } = require('@hoprnet/hopr-connect')
import pipe from 'it-pipe'

import { privKeyToPeerId, stringToU8a } from '@hoprnet/hopr-utils'

const id = '0x964e55c734330e9393a32aef61e6b75f3b526fb64df0ac55a6076045918657e1'

import Upgrader from 'libp2p/src/upgrader'
const yargs = require('yargs/yargs')
import { Multiaddr } from 'multiaddr'

// @ts-ignore
class MyUpgrader extends Upgrader {
  /**
   * A convenience method for generating a new `Connection`
   *
   * @override
   * @private
   * @param {object} options
   * @param {string} options.cryptoProtocol - The crypto protocol that was negotiated
   * @param {'inbound' | 'outbound'} options.direction - One of ['inbound', 'outbound']
   * @param {MultiaddrConnection} options.maConn - The transport layer connection
   * @param {MuxedStream | MultiaddrConnection} options.upgradedConn - A duplex connection returned from multiplexer and/or crypto selection
   * @param {MuxerFactory} [options.Muxer] - The muxer to be used for muxing
   * @param {PeerId} options.remotePeer - The peer the connection is with
   * @returns {Connection}
   */
  private _createConnection({ cryptoProtocol, direction, maConn, upgradedConn, Muxer, remotePeer }) {
    console.log(`inside`)
    /** @type {import("libp2p-interfaces/src/stream-muxer/types").Muxer} */
    let muxer
    /** @type {import("libp2p-interfaces/src/connection/connection").CreatedMuxedStream | undefined} */
    let newStream
    /** @type {Connection} */
    let connection // eslint-disable-line prefer-const

    if (Muxer) {
      // Create the muxer
      muxer = new Muxer({
        // Run anytime a remote stream is created
        onStream: async (muxedStream) => {
          if (!connection) return
          const mss = new Multistream.Listener(muxedStream)
          try {
            const { stream, protocol } = await mss.handle(Array.from(this.protocols.keys()))
            // log('%s: incoming stream opened on %s', direction, protocol)
            if (this.metrics) this.metrics.trackStream({ stream, remotePeer, protocol })
            connection.addStream(muxedStream, { protocol })
            // this._onStream({ connection, stream: { ...muxedStream, ...stream }, protocol })
          } catch (err) {
            // log.error(err)
          }
        },
        // Run anytime a stream closes
        onStreamEnd: (muxedStream) => {
          connection.removeStream(muxedStream.id)
        }
      })

      newStream = async (protocols) => {
        // log('%s: starting new stream on %s', direction, protocols)
        const muxedStream = muxer.newStream()
        const mss = new Multistream.Dialer(muxedStream)
        try {
          console.log(await mss.ls())
          // const { stream, protocol } = await mss.select(protocols)
          // if (this.metrics) this.metrics.trackStream({ stream, remotePeer, protocol })
          // return { stream: { ...muxedStream, ...stream }, protocol }
        } catch (err) {
          // log.error('could not create new stream', err)
          // throw errCode(err, codes.ERR_UNSUPPORTED_PROTOCOL)
        }
      }

      // Pipe all data through the muxer
      pipe(upgradedConn, muxer, upgradedConn) //.catch(log.error)
    }

    const _timeline = maConn.timeline
    maConn.timeline = new Proxy(_timeline, {
      set: (...args) => {
        if (connection && args[1] === 'close' && args[2] && !_timeline.close) {
          // Wait for close to finish before notifying of the closure
          ;(async () => {
            try {
              if (connection.stat.status === 'open') {
                await connection.close()
              }
            } catch (err) {
              //log.error(err)
            } finally {
              this.onConnectionEnd(connection)
            }
          })()
        }

        return Reflect.set(...args)
      }
    })
    maConn.timeline.upgraded = Date.now()

    // const errConnectionNotMultiplexed = () => {
    //   throw errCode(new Error('connection is not multiplexed'), 'ERR_CONNECTION_NOT_MULTIPLEXED')
    // }

    // // Create the connection
    // connection = new Connection({
    //   localAddr: maConn.localAddr,
    //   remoteAddr: maConn.remoteAddr,
    //   localPeer: this.localPeer,
    //   remotePeer: remotePeer,
    //   stat: {
    //     direction,
    //     // @ts-ignore
    //     timeline: maConn.timeline,
    //     multiplexer: Muxer && Muxer.multicodec,
    //     encryption: cryptoProtocol
    //   },
    //   newStream: newStream || errConnectionNotMultiplexed,
    //   getStreams: () => (muxer ? muxer.streams : errConnectionNotMultiplexed()),
    //   close: async () => {
    //     await maConn.close()
    //     // Ensure remaining streams are aborted
    //     if (muxer) {
    //       muxer.streams.map((stream) => stream.abort())
    //     }
    //   }
    // })

    // this.onConnection(connection)

    // return connection

    return {
      newStream
    }
  }
}

async function main() {
  const argv = yargs(process.argv.slice(2))
    .option('addr', {
      describe: 'example: --addr /ip4/34.65.42.178/tcp/9091/p2p/16Uiu2HAkyQGg2LLqwbDbuiHZSVtB3q5xmhpq7URirCEuJ4CXjZTh',
      type: 'string'
    })
    .parseSync()

  let ma: Multiaddr
  try {
    ma = new Multiaddr(argv.addr)
  } catch (err) {
    console.log(`Error while decoding Multiaddr: `, err)
  }

  const self = privKeyToPeerId(stringToU8a(id, 32))

  // @ts-ignore
  const upgrader = new MyUpgrader({
    localPeer: self,
    metrics: null
  })

  upgrader.cryptos.set(NOISE.protocol, NOISE)

  upgrader.muxers.set(MPLEX.multicodec, MPLEX)

  const Transport = new HoprConnect({
    upgrader,
    libp2p: {
      peerId: self,
      handle: () => {}
    }
  })

  await (await Transport.dial(ma)).newStream()
}

main()
