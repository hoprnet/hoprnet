import net, {AddressInfo, Socket as TCPSocket} from 'net'
import dgram, {RemoteInfo} from 'dgram'

import EventEmitter from 'events'
import debug from 'debug'

const log = debug('hopr-core:transport:listener')
const error = debug('hopr-core:transport:listener:error')
const verbose = debug('hopr-core:verbose:listener:error')

import type {Connection} from 'libp2p'
import {socketToConn} from './socket-to-conn'
import {CODE_P2P} from './constants'
import {MultiaddrConnection, Upgrader} from 'libp2p'
import Multiaddr from 'multiaddr'

import {handleStunRequest, getExternalIp} from './stun'
import {getAddrs} from './addrs'

const SOCKET_CLOSE_TIMEOUT = 400

/**
 * Attempts to close the given maConn. If a failure occurs, it will be logged.
 * @private
 * @param maConn
 */
async function attemptClose(maConn: MultiaddrConnection) {
  if (maConn == null) {
    return
  }

  try {
    await maConn.close()
  } catch (err) {
    error('an error occurred closing the connection', err)
  }
}

enum State {
  UNINITIALIZED,
  LISTENING,
  CLOSED
}

class Listener extends EventEmitter {
  private __connections: MultiaddrConnection[]
  private tcpSocket: net.Server
  private udpSocket: dgram.Socket

  private state: State

  private listeningAddr: Multiaddr
  private peerId: string

  private externalAddress: {
    address: string
    port: number
  }

  constructor(
    private handler: (conn: Connection) => void,
    private upgrader: Upgrader,
    private stunServers: Multiaddr[]
  ) {
    super()

    this.__connections = []
    this.upgrader = upgrader

    this.tcpSocket = net.createServer(this.onTCPConnection.bind(this))

    this.udpSocket = dgram.createSocket({
      // @TODO
      // `udp6` does not seem to work in Node 12.x
      // can receive IPv6 packet and IPv4 after reconnecting the socket
      type: 'udp4',
      reuseAddr: true
    })

    this.state = State.UNINITIALIZED

    Promise.all([
      new Promise((resolve) => this.udpSocket.once('listening', resolve)),
      new Promise((resolve) => this.tcpSocket.once('listening', resolve))
    ]).then(() => {
      this.state = State.LISTENING
      this.emit('listening')
    })

    Promise.all([
      new Promise((resolve) => this.udpSocket.once('close', resolve)),
      new Promise((resolve) => this.tcpSocket.once('close', resolve))
    ]).then(() => this.emit('close'))

    this.udpSocket.on('message', (msg: Buffer, rinfo: RemoteInfo) => handleStunRequest(this.udpSocket, msg, rinfo))

    this.tcpSocket.on('error', (err) => this.emit('error', err))
    this.udpSocket.on('error', (err) => this.emit('error', err))
  }

  async listen(ma: Multiaddr): Promise<void> {
    if (this.state == State.CLOSED) {
      throw Error(`Cannot listen after 'close()' has been called`)
    }

    this.listeningAddr = ma
    this.peerId = ma.getPeerId()

    if (this.peerId == null) {
      this.listeningAddr = ma.decapsulateCode(CODE_P2P)

      verbose(`No peerId for ${ma.toString()}`)
      if (!this.listeningAddr.isThinWaistAddress()) {
        throw Error(`Unable to bind socket to <${this.listeningAddr.toString()}>`)
      }
    }

    const options = this.listeningAddr.toOptions()

    // Prevent from sending a STUN request to ourself
    this.stunServers = this.stunServers?.filter((ma) => {
      const cOpts = ma.toOptions()

      return cOpts.host !== options.host || cOpts.port !== options.port
    })

    await Promise.all([
      new Promise((resolve, reject) =>
        this.tcpSocket.listen(options, (err?: Error) => {
          if (err) return reject(err)
          log('Listening on %s', this.tcpSocket.address())
          resolve()
        })
      ),
      new Promise((resolve) =>
        this.udpSocket.bind(options.port, async () => {
          try {
            this.externalAddress = await getExternalIp(this.stunServers, this.udpSocket)
          } catch (err) {
            error(`Unable to fetch external address using STUN. Error was: ${err}`)
          }

          resolve()
        })
      )
    ])

    this.state = State.LISTENING
  }

  async close(): Promise<void> {
    await Promise.all([
      new Promise((resolve) => {
        this.udpSocket.once('close', resolve)
        this.udpSocket.close()
      }),
      this.tcpSocket.listening
        ? new Promise((resolve) => {
            this.__connections.forEach(attemptClose)
            this.tcpSocket.once('close', resolve)
            this.tcpSocket.close()
          })
        : Promise.resolve()
    ])

    this.state = State.CLOSED

    // Give the operating system some time to release the sockets
    await new Promise((resolve) => setTimeout(resolve, SOCKET_CLOSE_TIMEOUT))
  }

  getAddrs() {
    if (this.state != State.LISTENING) {
      throw Error(`Listener is not yet ready`)
    }

    let addrs: Multiaddr[] = []
    const address = this.tcpSocket.address() as AddressInfo

    if (this.externalAddress != null && this.externalAddress.port == null) {
      log(`Attention: Bidirectional NAT detected. Publishing no public IPv4 address to the DHT`)

      addrs.push(Multiaddr(`/p2p/${this.peerId}`))

      addrs.push(
        ...getAddrs(address.port, this.peerId, {
          includeLocalhostIPv4: true
          // useIPv6: true
        })
      )
    } else if (this.externalAddress != null && this.externalAddress.port != null) {
      addrs.push(
        Multiaddr.fromNodeAddress(
          {
            ...this.externalAddress,
            family: 'IPv4',
            port: this.externalAddress.port.toString()
          },
          'tcp'
        ).encapsulate(`/p2p/${this.peerId}`)
      )

      addrs.push(
        ...getAddrs(address.port, this.peerId, {
          includeLocalhostIPv4: true
          // useIPv6: true
        })
      )
    } else {
      addrs.push(this.listeningAddr)
    }

    return addrs
  }

  private trackConn(maConn: MultiaddrConnection) {
    this.__connections.push(maConn)
    verbose(`currently tracking ${this.__connections.length} connections ++`)

    const untrackConn = () => {
      verbose(`currently tracking ${this.__connections.length} connections --`)
      this.__connections = this.__connections.filter((c: MultiaddrConnection) => c !== maConn)
    }

    maConn.conn.once('close', untrackConn)
  }

  private async onTCPConnection(socket: TCPSocket) {
    // Avoid uncaught errors caused by unstable connections
    socket.on('error', (err) => error('socket error', err))

    let maConn: MultiaddrConnection
    let conn: Connection
    try {
      maConn = socketToConn(socket, {listeningAddr: this.listeningAddr})
      log('new inbound connection %s', maConn.remoteAddr)
      conn = await this.upgrader.upgradeInbound(maConn)
    } catch (err) {
      error('inbound connection failed', err)
      return attemptClose(maConn)
    }

    log('inbound connection %s upgraded', maConn.remoteAddr)

    this.trackConn(maConn)

    this.handler?.(conn)

    this.emit('connection', conn)
  }
}

export default Listener
