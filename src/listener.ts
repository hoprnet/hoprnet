import net, { AddressInfo, Socket as TCPSocket } from 'net'
import dgram, { RemoteInfo } from 'dgram'

import { EventEmitter } from 'events'
import debug from 'debug'

import { socketToConn } from './socket-to-conn'
import { CODE_P2P } from './constants'
import type { Connection, ConnHandler } from 'libp2p'
import type { Listener as InterfaceListener } from 'libp2p-interfaces'
import type PeerId from 'peer-id'
import { MultiaddrConnection, Upgrader } from 'libp2p'
import Multiaddr from 'multiaddr'

import { handleStunRequest, getExternalIp } from './stun'
import { getAddrs } from './addrs'

const log = debug('hopr-connect:listener')
const error = debug('hopr-connect:listener:error')
const verbose = debug('hopr-connect:verbose:listener:error')

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

class Listener extends EventEmitter implements InterfaceListener {
  private __connections: MultiaddrConnection[]
  private tcpSocket: net.Server
  private udpSocket: dgram.Socket

  private state: State

  private listeningAddr?: Multiaddr

  private externalAddress?: {
    address: string
    port: number
  }

  constructor(
    private handler: ConnHandler | undefined,
    private upgrader: Upgrader,
    private stunServers: Multiaddr[] | undefined,
    private peerId: PeerId
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

    if (this.peerId.toB58String() !== ma.getPeerId()) {
      let tmpListeningAddr = ma.decapsulateCode(CODE_P2P)

      if (!tmpListeningAddr.isThinWaistAddress()) {
        throw Error(`Unable to bind socket to <${tmpListeningAddr.toString()}>`)
      }

      // Replace wrong PeerId in given listeningAddr
      this.listeningAddr = tmpListeningAddr.encapsulate(`/p2p/${this.peerId.toB58String()}`)
    } else {
      this.listeningAddr = ma
    }

    const options = this.listeningAddr.toOptions()

    // Prevent from sending a STUN request to ourself
    this.stunServers = this.stunServers?.filter((ma) => {
      const cOpts = ma.toOptions()

      return cOpts.host !== options.host || cOpts.port !== options.port
    })

    await Promise.all([
      new Promise<void>((resolve, reject) =>
        this.tcpSocket.listen(options, (err?: Error) => {
          if (err) return reject(err)
          log('Listening on %s', this.tcpSocket.address())
          resolve()
        })
      ),
      // @TODO handle socket bind error(s)
      new Promise<void>((resolve /*, reject*/) =>
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

    if (this.externalAddress == undefined) {
      log(`Attention: Bidirectional NAT detected. Publishing no public IPv4 address to the DHT`)

      addrs.push(Multiaddr(`/p2p/${this.peerId}`))

      addrs.push(
        ...getAddrs(address.port, this.peerId.toB58String(), {
          includeLocalhostIPv4: true,
          useIPv6: false
        })
      )
    } else {
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
        ...getAddrs(address.port, this.peerId.toB58String(), {
          includeLocalhostIPv4: true,
          useIPv6: false
        })
      )
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

    let maConn: MultiaddrConnection | undefined
    let conn: Connection
    try {
      maConn = socketToConn(socket, { listeningAddr: this.listeningAddr })
      log('new inbound connection %s', maConn.remoteAddr)
      conn = await this.upgrader.upgradeInbound(maConn)
    } catch (err) {
      error('inbound connection failed', err)

      if (maConn != undefined) {
        return attemptClose(maConn)
      }

      return
    }

    log('inbound connection %s upgraded', maConn.remoteAddr)

    this.trackConn(maConn)

    this.handler?.(conn)

    this.emit('connection', conn)
  }
}

export default Listener
