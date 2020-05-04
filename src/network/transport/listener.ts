import net, { AddressInfo } from 'net'
import EventEmitter from 'events'
import debug from 'debug'

const log = debug('libp2p:tcp:listener')
const error = debug('libp2p:tcp:listener:error')

import { socketToConn } from './socket-to-conn'
import { CODE_P2P } from './constants'
import { getMultiaddrs, multiaddrToNetConfig } from './utils'
import { MultiaddrConnection } from '.'
import Multiaddr from 'multiaddr'

export interface Listener extends EventEmitter {
  close(): void
  listen(ma: Multiaddr): Promise<void>
  getAddrs(): Multiaddr[]
}

/**
 * Attempts to close the given maConn. If a failure occurs, it will be logged.
 * @private
 * @param {MultiaddrConnection} maConn
 */
async function attemptClose(maConn: MultiaddrConnection) {
  try {
    maConn && (await maConn.close())
  } catch (err) {
    error('an error occurred closing the connection', err)
  }
}

export function createListener({ handler, upgrader }, options) {
  const listener = new EventEmitter() as Listener

  const server = net.createServer(async socket => {
    // Avoid uncaught errors caused by unstable connections
    socket.on('error', err => log('socket error', err))

    let maConn: MultiaddrConnection
    let conn
    try {
      maConn = socketToConn(socket, { listeningAddr })
      log('new inbound connection %s', maConn.remoteAddr)
      conn = await upgrader.upgradeInbound(maConn)
    } catch (err) {
      error('inbound connection failed', err)
      return attemptClose(maConn)
    }

    log('inbound connection %s upgraded', maConn.remoteAddr)

    trackConn(server, maConn)

    if (handler) {
      handler(conn)
    }

    listener.emit('connection', conn)
  })

  server
    .on('listening', () => listener.emit('listening'))
    .on('error', err => listener.emit('error', err))
    .on('close', () => listener.emit('close'))

  // Keep track of open connections to destroy in case of timeout
  // @ts-ignore
  server.__connections = []

  listener.close = () => {
    if (!server.listening) return

    return new Promise((resolve, reject) => {
      // @ts-ignore
      server.__connections.forEach(maConn => attemptClose(maConn))
      server.close(err => (err ? reject(err) : resolve()))
    })
  }

  let peerId, listeningAddr

  listener.listen = (ma: Multiaddr) => {
    listeningAddr = ma
    peerId = ma.getPeerId()

    if (peerId) {
      listeningAddr = ma.decapsulateCode(CODE_P2P)
    }

    return new Promise((resolve, reject) => {
      const options = multiaddrToNetConfig(listeningAddr)
      server.listen(options, (err?: Error) => {
        if (err) return reject(err)
        log('Listening on %s', server.address())
        resolve()
      })
    })
  }

  listener.getAddrs = () => {
    let addrs = []
    const address = server.address() as AddressInfo

    if (!address) {
      throw new Error('Listener is not ready yet')
    }

    // Because TCP will only return the IPv6 version
    // we need to capture from the passed multiaddr
    if (listeningAddr.toString().startsWith('/ip4')) {
      addrs = addrs.concat(getMultiaddrs('ip4', address.address, address.port))
    } else if (address.family === 'IPv6') {
      addrs = addrs.concat(getMultiaddrs('ip6', address.address, address.port))
    }

    return addrs.map(ma => (peerId ? ma.encapsulate(`/p2p/${peerId}`) : ma))
  }

  return listener
}

function trackConn(server, maConn) {
  server.__connections.push(maConn)

  const untrackConn = () => {
    server.__connections = server.__connections.filter(c => c !== maConn)
  }

  maConn.conn.once('close', untrackConn)
}
