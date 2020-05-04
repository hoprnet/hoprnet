import net from 'net'
import mafmt from 'mafmt'
const errCode = require('err-code')
const log = require('debug')('libp2p:tcp')
import { socketToConn } from './socket-to-conn'
import { createListener } from './listener'
import { multiaddrToNetConfig } from './utils'
import { AbortError } from 'abortable-iterator'
import { CODE_CIRCUIT, CODE_P2P } from './constants'

import type Multiaddr from 'multiaddr'
import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

interface DialOptions {
  signal?: AbortSignal
  relay?: PeerInfo | PeerId
}

export interface MultiaddrConnection {
  sink(source: any): void
  source: any
  close(err?: Error): Promise<void>
  conn: any
  remoteAddr: Multiaddr
  localAddr?: Multiaddr
  timeline: {
    open: number
    close?: number
  }
}

export interface Upgrader {
  upgradeOutbound(multiaddrConnection: MultiaddrConnection): Promise<any>
  upgradeInbound(multiaddrConnection: MultiaddrConnection): Promise<any>
}

/**
 * @class TCP
 */
class TCP {
  private _upgrader: Upgrader

  constructor({ upgrader }: { upgrader: Upgrader }) {
    if (!upgrader) {
      throw new Error(
        'An upgrader must be provided. See https://github.com/libp2p/interface-transport#upgrader.'
      )
    }
    this._upgrader = upgrader
  }

  /**
   * @async
   * @param {Multiaddr} ma
   * @param {object} options
   * @param {AbortSignal} options.signal Used to abort dial requests
   * @returns {Connection} An upgraded Connection
   */
  async dial(ma: Multiaddr, options?: DialOptions) {
    options = options || {}
    const socket = await this._connect(ma, options)
    const maConn = socketToConn(socket, { remoteAddr: ma, signal: options.signal })

    log('new outbound connection %s', maConn.remoteAddr)
    const conn = await this._upgrader.upgradeOutbound(maConn)

    log('outbound connection %s upgraded', maConn.remoteAddr)
    return conn
  }

  /**
   * @private
   * @param {Multiaddr} ma
   * @param {object} options
   * @param {AbortSignal} options.signal Used to abort dial requests
   * @returns {Promise<Socket>} Resolves a TCP Socket
   */
  _connect(ma: Multiaddr, options: DialOptions) {
    if (options.signal && options.signal.aborted) {
      throw new AbortError()
    }

    return new Promise((resolve, reject) => {
      const start = Date.now()
      const cOpts = multiaddrToNetConfig(ma) as any

      log('dialing %j', cOpts)
      const rawSocket = net.connect(cOpts)

      const onError = (err: Error) => {
        err.message = `connection error ${cOpts.host}:${cOpts.port}: ${err.message}`
        done(err)
      }

      const onTimeout = () => {
        log('connnection timeout %s:%s', cOpts.host, cOpts.port)
        const err = errCode(
          new Error(`connection timeout after ${Date.now() - start}ms`),
          'ERR_CONNECT_TIMEOUT'
        )
        // Note: this will result in onError() being called
        rawSocket.emit('error', err)
      }

      const onConnect = () => {
        log('connection opened %j', cOpts)
        done()
      }

      const onAbort = () => {
        log('connection aborted %j', cOpts)
        rawSocket.destroy()
        done(new AbortError())
      }

      const done = (err?: Error) => {
        rawSocket.removeListener('error', onError)
        rawSocket.removeListener('timeout', onTimeout)
        rawSocket.removeListener('connect', onConnect)
        options.signal && options.signal.removeEventListener('abort', onAbort)

        if (err) return reject(err)
        resolve(rawSocket)
      }

      rawSocket.on('error', onError)
      rawSocket.on('timeout', onTimeout)
      rawSocket.on('connect', onConnect)
      options.signal && options.signal.addEventListener('abort', onAbort)
    })
  }

  /**
   * Creates a TCP listener. The provided `handler` function will be called
   * anytime a new incoming Connection has been successfully upgraded via
   * `upgrader.upgradeInbound`.
   * @param {*} [options]
   * @param {function(Connection)} handler
   * @returns {Listener} A TCP listener
   */
  createListener(options, handler) {
    if (typeof options === 'function') {
      handler = options
      options = {}
    }
    options = options || {}
    return createListener({ handler, upgrader: this._upgrader }, options)
  }

  /**
   * Takes a list of `Multiaddr`s and returns only valid TCP addresses
   * @param {Multiaddr[]} multiaddrs
   * @returns {Multiaddr[]} Valid TCP multiaddrs
   */
  filter(multiaddrs) {
    multiaddrs = Array.isArray(multiaddrs) ? multiaddrs : [multiaddrs]

    return multiaddrs.filter(ma => {
      if (ma.protoCodes().includes(CODE_CIRCUIT)) {
        return false
      }

      return mafmt.TCP.matches(ma.decapsulateCode(CODE_P2P))
    })
  }
}

export default TCP
