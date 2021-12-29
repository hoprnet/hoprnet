import { createServer, type AddressInfo, type Socket as TCPSocket, type Server as TCPServer } from 'net'
import { createSocket, type RemoteInfo, type Socket as UDPSocket } from 'dgram'

import { once, EventEmitter } from 'events'
import type { PeerStoreType, PublicNodesEmitter } from '../types'
import Debug from 'debug'
import { networkInterfaces, type NetworkInterfaceInfo } from 'os'

import { CODE_P2P, CODE_IP4, CODE_IP6, CODE_TCP } from '../constants'
import type { MultiaddrConnection, Upgrader, Listener as InterfaceListener } from 'libp2p-interfaces/transport'

import type PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'

import { handleStunRequest, getExternalIp } from './stun'
import { getAddrs } from './addrs'
import { isAnyAddress } from '@hoprnet/hopr-utils'
import { TCPConnection } from './tcp'
import { EntryNodes } from './entry'
import { bindToPort, attemptClose } from '../utils'
import type HoprConnect from '..'

const log = Debug('hopr-connect:listener')
const error = Debug('hopr-connect:listener:error')
const verbose = Debug('hopr-connect:verbose:listener')

// @TODO to be adjusted
const SOCKET_CLOSE_TIMEOUT = 500

enum State {
  UNINITIALIZED,
  LISTENING,
  CLOSING,
  CLOSED
}

type Address = { port: number; address: string }

class Listener extends EventEmitter implements InterfaceListener {
  private __connections: MultiaddrConnection[]
  protected tcpSocket: TCPServer
  private udpSocket: UDPSocket

  private state: State
  private entry: EntryNodes
  private _emitListening: () => void

  private listeningAddr?: Multiaddr

  protected addrs: {
    interface: Multiaddr[]
    external: Multiaddr[]
    relays: Multiaddr[]
  }

  /**
   * @param handler called on incoming connection
   * @param upgrader inform libp2p about incoming connections
   * @param publicNodes emits on new and dead entry nodes
   * @param initialNodes array of entry nodes that is know at startup
   * @param peerId own id
   * @param _interface interface to listen on, e.g. eth0
   * @param __runningLocally [testing] treat local addresses as public addresses
   */
  constructor(
    dialDirectly: HoprConnect['dialDirectly'],
    private upgradeInbound: Upgrader['upgradeInbound'],
    publicNodes: PublicNodesEmitter | undefined,
    initialNodes: PeerStoreType[] = [],
    private peerId: PeerId,
    private _interface: string | undefined,
    private __runningLocally: boolean
  ) {
    super()

    this.__connections = []

    this.tcpSocket = createServer()
    this.udpSocket = createSocket({
      // @TODO
      // `udp6` does not seem to work in Node 12.x
      // can receive IPv6 packet and IPv4 after reconnecting the socket
      type: 'udp4',
      // set to true to reuse port that is bound
      // to TCP socket
      reuseAddr: true
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

    // Forward socket errors
    this.tcpSocket.on('error', (err) => this.emit('error', err))
    this.udpSocket.on('error', (err) => this.emit('error', err))

    this.tcpSocket.on('connection', async (socket: TCPSocket) => {
      try {
        await this.onTCPConnection(socket)
      } catch (err) {
        error(`network error`, err)
      }
    })
    this.udpSocket.on('message', (msg: Buffer, rinfo: RemoteInfo) => handleStunRequest(this.udpSocket, msg, rinfo))

    this.addrs = {
      interface: [],
      external: [],
      relays: []
    }

    this._emitListening = (() => this.emit('listening')).bind(this)

    this.entry = new EntryNodes(this.peerId, initialNodes, publicNodes, dialDirectly)
  }

  async bind(ma: Multiaddr) {
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

      // Replace wrong PeerId in given listeningAddr with own PeerId
      log(`replacing peerId in ${ma.toString()} by our peerId which is ${this.peerId.toB58String()}`)
      this.listeningAddr = tmpListeningAddr.encapsulate(`/p2p/${this.peerId.toB58String()}`)
    } else {
      this.listeningAddr = ma
    }

    const options = this.listeningAddr.toOptions()

    options.host = this.getAddressForInterface(options.host, family)

    if (options.port == 0 || options.port == null) {
      // First bind to any TCP port and then
      // bind the UDP socket and bind to same port
      await this.listenTCP().then((tcpPort) => this.listenUDP(tcpPort))
    } else {
      await Promise.all([
        // prettier-ignore
        this.listenTCP(options),
        this.listenUDP(options.port)
      ])
    }
  }

  /**
   * Attaches the listener to TCP and UDP sockets
   * @param ma address to listen to
   */
  async listen(ma: Multiaddr): Promise<void> {
    if (this.state == State.CLOSED) {
      throw Error(`Cannot listen after 'close()' has been called`)
    }

    await this.bind(ma)

    const address = this.tcpSocket.address() as AddressInfo

    this.addrs.interface.push(
      ...getAddrs(address.port, this.peerId.toB58String(), {
        useIPv4: true,
        includePrivateIPv4: true,
        includeLocalhostIPv4: true
      })
    )

    this.emit('listening')

    // Prevent from sending a STUN request to self
    let usableStunServers = this.getPotentialStunServers(address.port, address.address)
    await this.determinePublicIpAddress(usableStunServers)

    this.entry.on('relay:changed', this._emitListening)
    await this.entry.updatePublicNodes()

    this.entry.start()

    this.state = State.LISTENING
    // this.emit('listening')
  }

  /**
   * Closes the listener and closes underlying TCP and UDP sockets.
   * @dev ignores prematurely closed TCP sockets
   */
  async close(): Promise<void> {
    this.state = State.CLOSING

    this.entry.stop()
    this.entry.off('relay:changed', this._emitListening)

    await Promise.all([this.closeUDP(), this.closeTCP()])

    this.state = State.CLOSED
    this.emit('close')
  }

  /**
   * Used to determine which addresses to announce in the network.
   * @dev Should be called after `listen()` has returned
   * @dev List gets updated while waiting for `listen()`
   * @returns list of addresses under which the node is available
   */
  getAddrs(): Multiaddr[] {
    return [...this.addrs.external, ...this.entry.getUsedArrays(), ...this.addrs.interface]
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

      if ([index + 1, 1].includes(this.__connections.length)) {
        this.__connections.pop()
      } else {
        this.__connections[index] = this.__connections.pop() as MultiaddrConnection
      }
    }

    ;(maConn.conn as EventEmitter).once('close', untrackConn)
  }

  /**
   * Called on incoming TCP Connections. Initiates libp2p handshakes.
   * @param socket socket of incoming connection
   */
  private async onTCPConnection(socket: TCPSocket) {
    // Avoid uncaught errors caused by unstable connections
    socket.on('error', (err) => error('socket error', err))

    let maConn: MultiaddrConnection | undefined

    try {
      maConn = TCPConnection.fromSocket(socket, this.peerId) as any
    } catch (err: any) {
      error(`inbound connection failed. ${err.message}`)
    }

    if (maConn == undefined) {
      socket.destroy()
      return
    }

    log('new inbound connection %s', maConn.remoteAddr)

    try {
      await this.upgradeInbound(maConn)
    } catch (err: any) {
      if (err.code === 'ERR_ENCRYPTION_FAILED') {
        error(`inbound connection failed because encryption failed. Maybe connected to the wrong node?`)
      } else {
        error('inbound connection failed', err)
      }

      if (maConn != undefined) {
        return attemptClose(maConn, error)
      }

      return
    }

    log('inbound connection %s upgraded', maConn.remoteAddr)

    this.trackConn(maConn)
  }

  /**
   * Binds the process to a UDP socket
   * @param port binding port
   */
  private async listenUDP(port: number): Promise<number> {
    await bindToPort('UDP', this.udpSocket, error, { port })

    return this.udpSocket.address().port
  }

  /**
   * Binds the process to a TCP socket
   * @param opts host and port to bind to
   */
  private async listenTCP(opts?: { host?: string; port: number }): Promise<number> {
    await bindToPort('TCP', this.tcpSocket, error, opts)

    return (this.tcpSocket.address() as AddressInfo).port
  }

  /**
   * Closes the TCP socket and tries to close all pending
   * connections.
   * @returns Promise that resolves once TCP socket is closed
   */
  private async closeTCP() {
    if (!this.tcpSocket.listening) {
      return
    }

    await Promise.all(this.__connections.map((conn: MultiaddrConnection) => attemptClose(conn, error)))

    const promise = once(this.tcpSocket, 'close')

    this.tcpSocket.close()

    // Node.js bug workaround: ocassionally on macOS close is not emitted and callback is not called
    return Promise.race([
      promise,
      new Promise<void>((resolve) =>
        setTimeout(() => {
          resolve()
        }, SOCKET_CLOSE_TIMEOUT)
      )
    ])
  }

  /**
   * Closes the UDP socket
   * @returns Promise that resolves once UDP socket is closed
   */
  private closeUDP() {
    const promise = once(this.udpSocket, 'close')

    this.udpSocket.close()

    return promise
  }

  /**
   * Tries to determine a node's public IP address by
   * using STUN servers
   * @param port the port on which we are listening
   * @param host [optional] the host on which we are listening
   * @returns Promise that resolves once STUN request came back or STUN timeout was reched
   */
  private async determinePublicIpAddress(usableStunServers: Multiaddr[]): Promise<void> {
    let externalAddress: Address | undefined
    try {
      externalAddress = await getExternalIp(usableStunServers, this.udpSocket, this.__runningLocally)
    } catch (err: any) {
      error(`Determining public IP failed`, err.message)
      return
    }

    if (externalAddress == undefined) {
      log(`STUN requests led to multiple ambiguous results, node seems to be behind a bidirectional NAT.`)
      return
    }

    const externalMultiaddr = Multiaddr.fromNodeAddress(
      {
        address: externalAddress.address,
        port: externalAddress.port,
        family: 4
      },
      'tcp'
    ).encapsulate(`/p2p/${this.peerId}`)

    // Remove address but do not change address order
    for (let i = 0; i < this.addrs.interface.length; i++) {
      if (externalMultiaddr.equals(this.addrs.interface[i])) {
        this.addrs.interface.splice(i, 1)
      }
    }

    this.addrs.external.push(externalMultiaddr)
  }

  /**
   * Returns a list of STUN servers that we can use to determine
   * our own public IP address
   * @param port the port on which we are listening
   * @param host [optional] the host on which we are listening
   * @returns a list of STUN servers, excluding ourself
   */
  private getPotentialStunServers(port: number, host?: string): Multiaddr[] {
    const result = []

    const availableNodes = this.entry.getAvailabeEntryNodes()

    for (let i = 0; i < availableNodes.length; i++) {
      if (availableNodes[i].id.equals(this.peerId)) {
        continue
      }

      for (let j = 0; j < availableNodes[i].multiaddrs.length; j++) {
        let cOpts: { host: string; port: number }
        try {
          cOpts = availableNodes[i].multiaddrs[j].toOptions()
        } catch (err) {
          continue
        }

        if (cOpts.host === host && cOpts.port === port) {
          continue
        }

        result.push(availableNodes[i].multiaddrs[j])
      }
    }

    return result
  }

  private getAddressForInterface(host: string, family: NetworkInterfaceInfo['family']): string {
    if (this._interface == undefined) {
      return host
    }

    const osInterfaces = networkInterfaces()

    if (osInterfaces == undefined) {
      throw Error(`Machine seems to have no network interfaces.`)
    }

    if (osInterfaces[this._interface] == undefined) {
      throw Error(`Machine does not have requested interface ${this._interface}`)
    }

    const usableInterfaces = osInterfaces[this._interface]?.filter(
      (iface: NetworkInterfaceInfo) => iface.family == family && !iface.internal
    )

    if (usableInterfaces == undefined || usableInterfaces.length == 0) {
      throw Error(`Desired interface <${this._interface}> does not exist or does not have any external addresses.`)
    }

    const index = usableInterfaces.findIndex((iface) => host === iface.address)

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
}

export { Listener }
