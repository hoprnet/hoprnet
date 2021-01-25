/// <reference path="./@types/libp2p-interfaces.ts" />

import net, { AddressInfo, Socket as TCPSocket } from 'net'
import dgram, { RemoteInfo } from 'dgram'

import { EventEmitter } from 'events'
import debug from 'debug'
import { NetworkInterfaceInfo, networkInterfaces } from 'os'

import { CODE_P2P } from './constants'
import type { Connection, ConnHandler } from 'libp2p'
import type { Listener as InterfaceListener } from 'libp2p-interfaces'
import type PeerId from 'peer-id'
import { MultiaddrConnection, Upgrader } from 'libp2p'
import Multiaddr from 'multiaddr'

import { handleStunRequest, getExternalIp } from './stun'
import { getAddrs, isAnyAddress } from './addrs'
import { TCPConnection } from './tcp'

const log = debug('hopr-connect:listener')
const error = debug('hopr-connect:listener:error')
const verbose = debug('hopr-connect:verbose:listener')

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
    private peerId: PeerId,
    private _interface: string | undefined
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

    const protos = ma.protoNames()
    if (!['ip4', 'ip6'].includes(protos[0])) {
      throw Error(`Can only bind to IPv4 or IPv6 addresses`)
    }

    if (protos.length > 1 && protos[1] !== 'tcp') {
      throw Error(`Can only bind to TCP sockets`)
    }

    if (this.peerId.toB58String() !== ma.getPeerId()) {
      let tmpListeningAddr = ma.decapsulateCode(CODE_P2P)

      if (!tmpListeningAddr.isThinWaistAddress()) {
        throw Error(`Unable to bind socket to <${tmpListeningAddr.toString()}>`)
      }

      // Replace wrong PeerId in given listeningAddr
      log(`replacing peerId in ${ma.toString()} by our peerId which is ${this.peerId.toB58String()}`)
      this.listeningAddr = tmpListeningAddr.encapsulate(`/p2p/${this.peerId.toB58String()}`)
    } else {
      this.listeningAddr = ma
    }

    const options = this.listeningAddr.toOptions()

    if (this._interface != undefined) {
      const osInterface = networkInterfaces()[this._interface].filter(
        (iface: NetworkInterfaceInfo) => iface.family.toLowerCase() == options.family && !iface.internal
      )

      if (osInterface == undefined) {
        throw Error(`Desired interface <${this._interface}> does not exist or does not have any external addresses.`)
      }

      const index = osInterface.findIndex((iface) => options.host == iface.address)

      if (!isAnyAddress(ma) && index < 0) {
        throw Error(
          `Could not bind to interface ${this._interface} on address ${
            options.host
          } because it was configured with a different addresses: ${osInterface
            .map((iface) => iface.address)
            .join(`, `)}`
        )
      }

      // @TODO figure what to do if there is more than one address
      options.host = osInterface[0].address
    }

    // Prevent from sending a STUN request to ourself
    this.stunServers = this.stunServers?.filter((ma) => {
      const cOpts = ma.toOptions()

      return cOpts.host !== options.host || cOpts.port !== options.port
    })

    const listenTCP = (port?: number) =>
      new Promise<number>((resolve, reject) => {
        try {
          this.tcpSocket.listen(
            {
              host: options.host,
              port: port ?? options.port
            },
            (err?: Error) => {
              if (err) {
                return reject(err)
              }

              log('Listening on %s', this.tcpSocket.address())
              resolve((this.tcpSocket.address() as AddressInfo).port)
            }
          )
        } catch (err) {
          error(`Could bind to TCP socket. Error was: ${err.message}`)
          reject()
        }
      })

    const listenUDP = (port?: number) =>
      new Promise<number>((resolve, reject) => {
        this.udpSocket.once('error', reject)
        try {
          this.udpSocket.bind(
            {
              port: port ?? options.port
            },
            async () => {
              this.udpSocket.removeListener('error', reject)
              try {
                this.externalAddress = await getExternalIp(this.stunServers, this.udpSocket)
              } catch (err) {
                error(`Unable to fetch external address using STUN. Error was: ${err}`)
              }

              resolve(this.udpSocket.address().port)
            }
          )
        } catch (err) {
          error(`Could bind UDP socket. Error was: ${err.message}`)
          reject()
        }
      })

    if (options.port == 0 || options.port == null) {
      await listenTCP().then((port) => listenUDP(port))
    } else {
      await Promise.all([listenTCP(), listenUDP()])
    }

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
    }

    addrs.push(Multiaddr(`/p2p/${this.peerId}`))

    addrs.push(
      ...getAddrs(address.port, this.peerId.toB58String(), {
        includeLocalIPv4: true,
        includeLocalhostIPv4: true,
        useIPv6: false
      })
    )

    return addrs
  }

  private trackConn(maConn: MultiaddrConnection) {
    this.__connections.push(maConn)
    verbose(`currently tracking ${this.__connections.length} connections ++`)

    const untrackConn = () => {
      verbose(`currently tracking ${this.__connections.length} connections --`)
      let index = this.__connections.findIndex((c: MultiaddrConnection) => c !== maConn)

      if ([index, 1].includes(this.__connections.length)) {
        this.__connections.pop()
      } else {
        this.__connections[index] = this.__connections.pop() as MultiaddrConnection
      }
    }

    maConn.conn.once('close', untrackConn)
  }

  private async onTCPConnection(socket: TCPSocket) {
    // Avoid uncaught errors caused by unstable connections
    socket.on('error', (err) => error('socket error', err))

    let maConn: MultiaddrConnection | undefined
    let conn: Connection
    try {
      maConn = TCPConnection.fromSocket(socket)
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
