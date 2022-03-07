import {
  createServer,
  createConnection,
  type AddressInfo,
  type Socket as TCPSocket,
  type Server as TCPServer
} from 'net'
import { createSocket, type RemoteInfo, type Socket as UDPSocket } from 'dgram'

import { once, EventEmitter } from 'events'
import type { PeerStoreType, HoprConnectOptions, HoprConnectTestingOptions } from '../types'
import Debug from 'debug'
import { networkInterfaces, type NetworkInterfaceInfo } from 'os'

import { CODE_P2P, CODE_IP4, CODE_IP6, CODE_TCP } from '../constants'
import type {
  MultiaddrConnection,
  Upgrader,
  Listener as InterfaceListener
} from 'libp2p-interfaces/src/transport/types'

import PeerId from 'peer-id'
import { Multiaddr } from 'multiaddr'

import { handleStunRequest, getExternalIp } from './stun'
import { getAddrs } from './addrs'
import { isAnyAddress, u8aEquals, defer } from '@hoprnet/hopr-utils'
import { TCPConnection } from './tcp'
import { EntryNodes, RELAY_CHANGED_EVENT } from './entry'
import { bindToPort, attemptClose, nodeToMultiaddr } from '../utils'
import type HoprConnect from '..'
import { UpnpManager } from './upnp'
import type { Filter } from '../filter'
import type { Relay } from '../relay'

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
  protected __connections: MultiaddrConnection[]
  protected tcpSocket: TCPServer
  private udpSocket: UDPSocket
  private upnpManager: UpnpManager

  private state: State
  private entry: EntryNodes
  private _emitListening: () => void

  private listeningAddr?: Multiaddr

  protected addrs: {
    interface: Multiaddr[]
    external: Multiaddr[]
  }

  /**
   * @param handler called on incoming connection
   * @param upgrader inform libp2p about incoming connections
   * @param publicNodes emits on new and dead entry nodes
   * @param initialNodes array of entry nodes that is know at startup
   * @param peerId own id
   * @param iface interface to listen on, e.g. `eth0`
   * @param __testingOptions.runningLocally [testing] assume that all nodes are running on localhost
   * @param __testingOptions.preferLocalAddresses [testing] treat local addresses as public addresses
   * @param __testingOptions.noUPNP [testing] disable UPNP support, speedup calls to checkNATSituation
   */
  constructor(
    dialDirectly: HoprConnect['dialDirectly'],
    private upgradeInbound: Upgrader['upgradeInbound'],
    private peerId: PeerId,
    private options: HoprConnectOptions,
    private testingOptions: HoprConnectTestingOptions,
    private filter: Filter,
    private relay: Relay
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

    this.addrs = {
      interface: [],
      external: []
    }

    this._emitListening = function (this: Listener) {
      // hopr-connect does not enable IPv6 connections right now, therefore we can set `listeningAddrs` statically
      // to `/ip4/0.0.0.0/tcp/0`, meaning listening on IPv4 using a canonical port
      // TODO check IPv6
      this.filter.setAddrs(this.getAddrs(), [new Multiaddr(`/ip4/0.0.0.0/tcp/0/p2p/${this.peerId.toB58String()}`)])

      const usedRelays = this.entry.getUsedRelays()

      if (usedRelays && usedRelays.length > 0) {
        const relayPeerIds = this.entry.getUsedRelays().map((ma: Multiaddr) => {
          const tuples = ma.tuples()

          return PeerId.createFromBytes((tuples[0][1] as any).slice(1))
        })

        this.relay.setUsedRelays(relayPeerIds)
      }

      this.emit('listening')
    }.bind(this)

    this.entry = new EntryNodes(this.peerId, dialDirectly, this.options)

    this.upnpManager = new UpnpManager()
  }

  attachSocketHandlers() {
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
  }

  async bind(ma: Multiaddr): Promise<void> {
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

  async isExposedHost(
    externalIp: string,
    port: number
  ): Promise<{
    udpMapped: boolean
    tcpMapped: boolean
  }> {
    const UDP_TEST = new TextEncoder().encode('TEST_UDP')
    const TCP_TEST = new TextEncoder().encode('TEST_TCP')

    const waitForIncomingUdpPacket = defer<void>()
    const waitForIncomingTcpPacket = defer<void>()

    const TIMEOUT = 500

    const abort = new AbortController()
    const tcpTimeout = setTimeout(() => {
      abort.abort()
      waitForIncomingTcpPacket.reject()
    }, TIMEOUT)
    const udpTimeout = setTimeout(waitForIncomingUdpPacket.reject.bind(waitForIncomingUdpPacket), TIMEOUT)

    const checkTcpMessage = (socket: TCPSocket) => {
      socket.on('data', (data: Buffer) => {
        if (u8aEquals(data, TCP_TEST)) {
          clearTimeout(tcpTimeout)
          waitForIncomingTcpPacket.resolve()
        }
      })
    }
    this.tcpSocket.on('connection', checkTcpMessage)

    const checkUdpMessage = (msg: Buffer) => {
      if (u8aEquals(msg, UDP_TEST)) {
        clearTimeout(udpTimeout)
        waitForIncomingUdpPacket.resolve()
      }
    }
    this.udpSocket.on('message', checkUdpMessage)

    const secondUdpSocket = createSocket('udp4')
    secondUdpSocket.send(UDP_TEST, port, externalIp)

    let done = false
    const cleanUp = (): void => {
      if (done) {
        return
      }
      done = true
      clearTimeout(tcpTimeout)
      clearTimeout(udpTimeout)
      this.udpSocket.removeListener('message', checkUdpMessage)
      this.tcpSocket.removeListener('connection', checkTcpMessage)
      tcpSocket.destroy()
      secondUdpSocket.close()
    }

    const tcpSocket = createConnection({
      port,
      host: externalIp,
      signal: abort.signal
    })
      .on('connect', () => {
        tcpSocket.write(TCP_TEST, (err: any) => {
          if (err) {
            log(`Failed to send TCP packet`, err)
          }
        })
      })
      .on('error', (err: any) => {
        if (err && (err.code == undefined || err.code !== 'ABORT_ERR')) {
          error(`Error while checking NAT situation`, err.message)
        }
      })

    if (!done) {
      const results = await Promise.allSettled([waitForIncomingUdpPacket.promise, waitForIncomingTcpPacket.promise])

      cleanUp()

      return {
        udpMapped: results[0].status === 'fulfilled',
        tcpMapped: results[1].status === 'fulfilled'
      }
    }

    return {
      udpMapped: false,
      tcpMapped: false
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

    const ownInterface = this.tcpSocket.address() as AddressInfo

    const natSituation = await this.checkNATSituation(ownInterface.address, ownInterface.port)

    log(`NAT situation detected: `, natSituation)
    const internalInterfaces = getAddrs(ownInterface.port, {
      useIPv4: true,
      includePrivateIPv4: true,
      includeLocalhostIPv4: true
    })

    if (!natSituation.bidirectionalNAT) {
      // If any of the interface addresses is the
      // external address,
      for (const [index, internalInterface] of internalInterfaces.entries()) {
        if (internalInterface.address == natSituation.externalAddress) {
          internalInterfaces.splice(index, 1)
        }
      }

      this.addrs.external = [
        nodeToMultiaddr(
          {
            address: natSituation.externalAddress,
            port: natSituation.externalPort,
            family: 'IPv4'
          },
          this.peerId
        )
      ]
    }

    this.addrs.interface = internalInterfaces.map((internalInterface) =>
      nodeToMultiaddr(internalInterface, this.peerId)
    )

    this.attachSocketHandlers()

    this._emitListening()

    // Only add relay nodes if node is not directly reachable or running locally
    if (this.testingOptions.__runningLocally || natSituation.bidirectionalNAT || !natSituation.isExposed) {
      this.entry.on(RELAY_CHANGED_EVENT, this._emitListening)

      // Finish startup
      this.entry.start()

      await this.entry.updatePublicNodes()
    }

    this.state = State.LISTENING
  }

  /**
   * Closes the listener and closes underlying TCP and UDP sockets.
   * @dev ignores prematurely closed TCP sockets
   */
  async close(): Promise<void> {
    this.state = State.CLOSING

    // Remove listeners
    this.entry.stop()
    this.entry.off(RELAY_CHANGED_EVENT, this._emitListening)

    await Promise.all([
      // Unmap all mapped UPNP ports and release socket
      this.upnpManager.stop(),
      this.closeUDP(),
      this.closeTCP()
    ])

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
    return [...this.addrs.external, ...this.entry.getUsedRelays(), ...this.addrs.interface]
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
  async checkNATSituation(
    ownAddress: string,
    ownPort: number
  ): Promise<
    | { bidirectionalNAT: true }
    | { bidirectionalNAT: false; externalAddress: string; externalPort: number; isExposed: boolean }
  > {
    if (this.testingOptions.__runningLocally) {
      const address = this.tcpSocket.address() as Address

      // Pretend to be an exposed host if running locally, e.g. as part of an E2E test
      return {
        bidirectionalNAT: false,
        externalAddress: address.address,
        externalPort: address.port,
        isExposed: true
      }
    }
    let externalAddress = this.testingOptions.__noUPNP ? undefined : await this.upnpManager.externalIp()
    let externalPort: number | undefined

    let isExposedHost: Awaited<ReturnType<Listener['isExposedHost']>> | undefined
    if (externalAddress != undefined) {
      // UPnP is supported, let's try to open the port
      await this.upnpManager.map(ownPort)

      // Don't trust the router blindly ...
      isExposedHost = await this.isExposedHost(externalAddress, ownPort)

      if (isExposedHost.tcpMapped || isExposedHost.udpMapped) {
        // Either TCP or UDP were mapped
        externalPort = ownPort
      } else {
        // Neither TCP nor UDP were reachable, maybe external IP / Port is wrong
        // fallback to STUN to get better results
        const usableStunServers = this.getUsableStunServers(ownAddress, ownPort)

        let externalInterface: Address | undefined
        try {
          externalInterface = await getExternalIp(
            usableStunServers,
            this.udpSocket,
            this.testingOptions.__preferLocalAddresses
          )
        } catch (err: any) {
          error(`Determining public IP failed`, err.message)
        }

        if (externalInterface != undefined) {
          externalPort = externalInterface.port

          isExposedHost = await this.isExposedHost(externalAddress, externalPort)
        }
      }
    } else {
      // UPnP is not supported, fallback to STUN
      const usableStunServers = this.getUsableStunServers(ownAddress, ownPort)

      let externalInterface: Address | undefined
      try {
        externalInterface = await getExternalIp(
          usableStunServers,
          this.udpSocket,
          this.testingOptions.__preferLocalAddresses
        )
      } catch (err: any) {
        error(`Determining public IP failed`, err.message)
      }

      if (externalInterface != undefined) {
        externalPort = externalInterface.port
        externalAddress = externalInterface.address

        isExposedHost = await this.isExposedHost(externalInterface.address, externalInterface.port)
      }
    }

    if (externalAddress && externalPort) {
      return {
        externalAddress,
        externalPort,
        // If we don't allow direct connections, then the host can obviously
        // not be considered to be exposed
        isExposed: this.testingOptions.__noDirectConnections
          ? false
          : (isExposedHost?.udpMapped && isExposedHost?.tcpMapped) ?? false,
        bidirectionalNAT: false
      }
    }

    return {
      bidirectionalNAT: true
    }
  }

  /**
   * Returns a list of STUN servers that we can use to determine
   * our own public IP address
   * @param ownPort the port on which we are listening
   * @param ownHost [optional] the host on which we are listening
   * @returns a list of STUN servers, excluding ourself
   */
  private getUsableStunServers(ownHost: string, ownPort: number): Multiaddr[] {
    const filtered = []

    let usableNodes: PeerStoreType[] = this.entry.getAvailabeEntryNodes()

    if (usableNodes.length == 0) {
      // Use unchecked nodes at startup
      usableNodes = this.entry.getUncheckedEntryNodes()
    }

    for (const usableNode of usableNodes) {
      if (usableNode.id.equals(this.peerId)) {
        // Exclude self
        continue
      }

      for (const multiaddr of usableNode.multiaddrs) {
        let cOpts: { host: string; port: number }
        try {
          cOpts = multiaddr.toOptions()
        } catch (err) {
          // Exclude unusable Multiaddrs
          continue
        }

        if (cOpts.host === ownHost && cOpts.port === ownPort) {
          // Exclude self
          continue
        }

        filtered.push(multiaddr)
      }
    }

    return filtered
  }

  private getAddressForInterface(host: string, family: NetworkInterfaceInfo['family']): string {
    if (this.options.interface == undefined) {
      return host
    }

    const osInterfaces = networkInterfaces()

    if (osInterfaces == undefined) {
      throw Error(`Machine seems to have no network interfaces.`)
    }

    if (osInterfaces[this.options.interface] == undefined) {
      throw Error(`Machine does not have requested interface ${this.options.interface}`)
    }

    const usableInterfaces = osInterfaces[this.options.interface]?.filter(
      (iface: NetworkInterfaceInfo) => iface.family == family && !iface.internal
    )

    if (usableInterfaces == undefined || usableInterfaces.length == 0) {
      throw Error(
        `Desired interface <${this.options.interface}> does not exist or does not have any external addresses.`
      )
    }

    const index = usableInterfaces.findIndex((iface) => host === iface.address)

    if (!isAnyAddress(host, family) && index < 0) {
      throw Error(
        `Could not bind to interface ${
          this.options.interface
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
