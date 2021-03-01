/// <reference path="./@types/libp2p.ts" />
/// <reference path="./@types/libp2p-interfaces.ts" />

import net, { AddressInfo, Socket as TCPSocket } from 'net'
import dgram, { RemoteInfo } from 'dgram'

import { EventEmitter } from 'events'
import debug from 'debug'
import { NetworkInterfaceInfo, networkInterfaces } from 'os'

import AbortController from 'abort-controller'
import type { AbortSignal } from 'abort-controller'
import { CODE_P2P, CODE_IP4, CODE_IP6, CODE_TCP, RELAY_CONTACT_TIMEOUT } from './constants'
import type { Connection, ConnHandler } from 'libp2p'
import { MultiaddrConnection, Upgrader } from 'libp2p'

import type { Listener as InterfaceListener } from 'libp2p-interfaces'
import type PeerId from 'peer-id'
import Multiaddr from 'multiaddr'

import { handleStunRequest, getExternalIp } from './stun'
import { getAddrs } from './addrs'
import { isAnyAddress } from './utils'
import { TCPConnection } from './tcp'

const log = debug('hopr-connect:listener')
const error = debug('hopr-connect:listener:error')
const verbose = debug('hopr-connect:verbose:listener')

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
  CLOSING,
  CLOSED
}

type ConnectResult = { id: string; latency: number }

class Listener extends EventEmitter implements InterfaceListener {
  private __connections: MultiaddrConnection[]
  private tcpSocket: net.Server
  private udpSocket: dgram.Socket

  private state: State

  private listeningAddr?: Multiaddr

  private relayConnectResults?: ConnectResult[]

  private externalAddress?: {
    address: string
    port: number
  }

  private stunServers: Multiaddr[] | undefined

  constructor(
    private handler: ConnHandler | undefined,
    private upgrader: Upgrader,
    stunServers: Multiaddr[] | undefined,
    private relays: Multiaddr[] | undefined,
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

    this.stunServers = stunServers?.filter((ma: Multiaddr) => {
      let maPeerId: string
      try {
        maPeerId = ma.getPeerId()
      } catch (err) {
        // Allow STUN server without a PeerId
        return true
      }

      // Do not self as STUN server
      return maPeerId !== this.peerId.toB58String()
    })

    this.state = State.UNINITIALIZED

    this.udpSocket.once('close', () => {
      if (![State.CLOSING, State.CLOSED].includes(this.state)) {
        console.trace(`UDP socket was closed earlier than expected. Please report this!`)
      }
    })

    this.tcpSocket.once('close', () => {
      if (![State.CLOSING, State.CLOSED].includes(this.state)) {
        console.trace(`TCP socket was closed earlier than expected. Please report this!`)
      }
    })

    this.udpSocket.on('message', (msg: Buffer, rinfo: RemoteInfo) => handleStunRequest(this.udpSocket, msg, rinfo))

    // Forward socket errors
    this.tcpSocket.on('error', (err) => this.emit('error', err))
    this.udpSocket.on('error', (err) => this.emit('error', err))
  }

  /**
   * Attaches the listener to TCP and UDP sockets
   * @param ma address to listen to
   */
  async listen(ma: Multiaddr): Promise<void> {
    if (this.state == State.CLOSED) {
      throw Error(`Cannot listen after 'close()' has been called`)
    }

    const protos = ma.tuples()
    let family: NetworkInterfaceInfo['family']

    switch (protos[0][0]) {
      case CODE_IP4:
        family = 'IPv4'
        break
      case CODE_IP6:
        family = 'IPv6'
        break
      default:
        throw Error(`Can only bind to IPv4 or IPv6 addresses`)
    }

    if (protos.length > 1 && protos[1][0] != CODE_TCP) {
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

    options.host = this.getAddressForInterface(options.host, family)

    if (options.port == 0 || options.port == null) {
      // @TODO check listening to host on any port
      const tcpPort = await this.listenTCP()
      await this.listenUDP(tcpPort)
    } else {
      await Promise.all([
        // prettier-ignore
        this.listenTCP(options),
        this.listenUDP(options.port, options.host)
      ])
    }

    await this.connectToRelays()

    this.state = State.LISTENING
    this.emit('listening')
  }

  /**
   * Closes the listener and closes underlying TCP and UDP sockets.
   * @dev ignores prematurely closed TCP sockets
   */
  async close(): Promise<void> {
    this.state = State.CLOSING

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

    this.emit('close')
  }

  /**
   * Used to determine which addresses to announce in the network.
   * @dev Called after `listen()` has returned
   */
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

    for (const res of this.relayConnectResults ?? []) {
      addrs.push(Multiaddr(`/p2p/${res.id}/p2p-circuit/p2p/${this.peerId}`))
    }

    addrs.push(
      ...getAddrs(address.port, this.peerId.toB58String(), {
        includeLocalIPv4: true,
        includeLocalhostIPv4: true,
        useIPv6: false
      })
    )

    return addrs
  }

  /**
   * Get listening port
   * @dev used for testing
   */
  getPort(): number {
    return (this.tcpSocket.address() as AddressInfo)?.port
  }

  /**
   * Tracks connections to close them once necessary.
   * @param maConn connection to track
   */
  private trackConn(maConn: MultiaddrConnection) {
    this.__connections.push(maConn)
    verbose(`currently tracking ${this.__connections.length} connections ++`)

    const untrackConn = () => {
      verbose(`currently tracking ${this.__connections.length} connections --`)
      let index = this.__connections.findIndex((c: MultiaddrConnection) => c === maConn)

      if (index < 0) {
        // connection not found
        verbose(`DEBUG: Connection not found.`, maConn)
        return
      }

      if ([index, 1].includes(this.__connections.length)) {
        this.__connections.pop()
      } else {
        this.__connections[index] = this.__connections.pop() as MultiaddrConnection
      }
    }

    maConn.conn.once('close', untrackConn)
  }

  /**
   * Called on incoming TCP Connections. Initiates libp2p handshakes.
   * @param socket socket of incoming connection
   */
  private async onTCPConnection(socket: TCPSocket) {
    // Avoid uncaught errors caused by unstable connections
    socket.on('error', (err) => error('socket error', err))

    let maConn: MultiaddrConnection | undefined
    let conn: Connection
    try {
      maConn = TCPConnection.fromSocket(socket, this.peerId)
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

  /**
   * Binds the process to a UDP socket and uses the socket
   * to retrieve the public IP address by using STUN
   * @param port binding port
   */
  private async listenUDP(port: number, host?: string): Promise<number> {
    await Promise.all([
      new Promise<void>((resolve) => this.udpSocket.once('listening', resolve)),
      new Promise<void>((resolve, reject) => {
        this.udpSocket.once('error', reject)

        this.udpSocket.bind(port, () => {
          this.udpSocket.removeListener('error', reject)
          resolve()
        })
      })
    ])

    // Prevent from sending a STUN request to self
    const usableStunServers =
      host == undefined
        ? this.stunServers
        : this.stunServers?.filter((ma) => {
            let cOpts: { host: string; port: number }
            try {
              cOpts = ma.toOptions()
            } catch (err) {
              return false
            }

            return cOpts.host !== host || cOpts.port !== port
          })

    this.externalAddress = await getExternalIp(usableStunServers, this.udpSocket)

    return this.udpSocket.address().port
  }

  /**
   * Binds the process to a TCP socket
   * @param opts host and port to bind to
   */
  private async listenTCP(opts?: { host: string; port: number }): Promise<number> {
    await Promise.all([
      new Promise<void>((resolve) => this.tcpSocket.once('listening', resolve)),
      new Promise<void>((resolve, reject) => {
        this.tcpSocket.once('error', reject)

        try {
          this.tcpSocket.listen(opts, () => {
            this.tcpSocket.removeListener('error', reject)
            resolve()
          })
        } catch (err) {
          error(`Could not bind to TCP socket. ${err.message}`)
          reject()
        }
      })
    ])

    log('Listening on %s', this.tcpSocket.address())

    return (this.tcpSocket.address() as AddressInfo).port
  }

  private getAddressForInterface(host: string, family: NetworkInterfaceInfo['family']): string {
    if (this._interface == undefined) {
      return host
    }

    const osInterfaces = networkInterfaces()

    if (osInterfaces == undefined) {
      throw Error(`Machine seems to have no networkInterfaces.`)
    }

    if (osInterfaces[this._interface] == undefined) {
      throw Error(`Machine does not have requested interface ${this._interface}`)
    }

    const usableInterfaces = osInterfaces[this._interface]?.filter(
      (iface: NetworkInterfaceInfo) => iface.family.toLowerCase() == family && !iface.internal
    )

    if (usableInterfaces == undefined) {
      throw Error(`Desired interface <${this._interface}> does not exist or does not have any external addresses.`)
    }

    const index = usableInterfaces.findIndex((iface) => host == iface.address)

    if (!isAnyAddress(host, family) && index < 0) {
      throw Error(
        `Could not bind to interface ${
          this._interface
        } on address ${host} because it was configured with a different addresses: ${usableInterfaces
          .map((iface) => iface.address)
          .join(`, `)}`
      )
    }

    // @TODO figure what to do if there is more than one address
    return usableInterfaces[0].address
  }

  private async connectToRelays() {
    // @TODO check address family
    if (this.relays == undefined || this.relays.length == 0) {
      return
    }

    const promises: Promise<ConnectResult>[] = []
    const abort = new AbortController()

    const timeout = setTimeout(abort.abort.bind(abort), RELAY_CONTACT_TIMEOUT)

    for (const relay of this.relays) {
      const relayPeerId = relay.getPeerId()

      if (relayPeerId == null || this.peerId.toB58String() === relayPeerId) {
        // Relay must have a peerId and must be not self
        continue
      }

      promises.push(this.connectToRelay(relay, relayPeerId, { signal: abort.signal }))
    }

    const results = await Promise.all(promises)

    clearTimeout(timeout)

    this.relayConnectResults = results.filter((res) => res.latency >= 0).sort((a, b) => a.latency - b.latency)
  }

  private async connectToRelay(relay: Multiaddr, relayPeerId: string, opts?: { signal: AbortSignal }) {
    const start = Date.now()
    let latency: number
    let conn: Connection | undefined
    let maConn: MultiaddrConnection | undefined

    try {
      maConn = await TCPConnection.create(relay, this.peerId, opts)
    } catch (err) {
      if (maConn != undefined) {
        attemptClose(maConn)
      }

      return {
        id: relayPeerId,
        latency: -1
      }
    }

    if (maConn == undefined) {
      return {
        id: relayPeerId,
        latency: -1
      }
    }

    try {
      conn = await this.upgrader.upgradeOutbound(maConn)
      latency = Date.now() - start
    } catch (err) {
      console.log(err)
      attemptClose(maConn)

      return {
        id: relayPeerId,
        latency: -1
      }
    }

    if (conn == undefined) {
      attemptClose(maConn)

      return {
        id: relayPeerId,
        latency: -1
      }
    }

    this.trackConn(maConn)

    this.handler?.(conn)

    this.emit('connection', conn)

    return {
      id: relayPeerId,
      latency
    }
  }
}

export default Listener
