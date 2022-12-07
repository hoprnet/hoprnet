import type { Connection, MultiaddrConnection } from '@libp2p/interface-connection'
import type { Listener as InterfaceListener, ListenerEvents } from '@libp2p/interface-transport'
import { EventEmitter, CustomEvent } from '@libp2p/interfaces/events'

import { networkInterfaces, type NetworkInterfaceInfo } from 'os'
import { createServer, type AddressInfo, type Socket as TCPSocket, type Server as TCPServer } from 'net'
import { createSocket, type RemoteInfo, type Socket as UDPSocket } from 'dgram'
import { once } from 'events'

import Debug from 'debug'
import { peerIdFromBytes } from '@libp2p/peer-id'
import { Multiaddr } from '@multiformats/multiaddr'

import { isAnyAddress, randomInteger, retimer } from '@hoprnet/hopr-utils'

import { CODE_P2P, CODE_IP4, CODE_IP6, CODE_TCP } from '../constants.js'
import {
  type PeerStoreType,
  type HoprConnectOptions,
  type HoprConnectTestingOptions,
  PeerConnectionType
} from '../types.js'
import { handleUdpStunRequest, getExternalIp, isExposedHost, handleTcpStunRequest } from './stun/index.js'
import { getAddrs } from './addrs.js'
import { fromSocket } from './tcp.js'
import { RELAY_CHANGED_EVENT } from './entry.js'
import { bindToPort, attemptClose, nodeToMultiaddr, cleanExistingConnections, ip6Lookup } from '../utils/index.js'

import isStun from 'is-stun'

import type { Components } from '@libp2p/interfaces/components'
import type { ConnectComponents } from '../components.js'

const log = Debug('hopr-connect:listener')
const error = Debug('hopr-connect:listener:error')
const verbose = Debug('hopr-connect:verbose:listener')

// @TODO to be adjusted
const SOCKET_CLOSE_TIMEOUT = 500

enum ListenerState {
  UNINITIALIZED,
  LISTENING,
  CLOSING,
  CLOSED
}

type Address = { port: number; address: string }

type NATSituation =
  | { bidirectionalNAT: true }
  | { bidirectionalNAT: false; externalAddress: string; externalPort: number; isExposed: boolean }

export type ProtocolListener = {
  identifier: string
  isProtocol: (msg: Buffer) => boolean
  takeStream: (socket: TCPSocket, stream: AsyncIterable<Uint8Array>) => void
}

// @ts-ignore libp2p interfaces type clash
class Listener extends EventEmitter<ListenerEvents> implements InterfaceListener {
  protected __connections: Connection[]
  protected tcpSocket: TCPServer

  private stopUdpSocketKeepAliveInterval: (() => void) | undefined
  private udpSocket: UDPSocket

  private protocols: ProtocolListener[]

  private state: ListenerState
  private _emitListening: () => void

  private listeningAddr?: Multiaddr

  protected addrs: {
    interface: Multiaddr[]
    external: Multiaddr[]
  }

  /**
   * @param options connection Options, e.g. AbortSignal
   * @param testingOptions turn on / off modules for testing
   * @param components Libp2p instance components
   * @param connectComponents HoprConnect components
   */
  constructor(
    private options: HoprConnectOptions,
    private testingOptions: HoprConnectTestingOptions,
    private components: Components,
    private connectComponents: ConnectComponents
  ) {
    super()

    this.__connections = []

    this.tcpSocket = createServer()
    this.udpSocket = createSocket({
      // `udp4` seems to have binding issues
      type: 'udp6',
      // set to true to use same port for TCP and UDP
      reuseAddr: true,
      // We use IPv4 traffic on udp6 sockets, so DNS queries
      // must return the A record (IPv4) not the AAAA record (IPv6)
      // - unless we explicitly check for a IPv6 address
      lookup: ip6Lookup
    })

    this.state = ListenerState.UNINITIALIZED

    this.addrs = {
      interface: [],
      external: []
    }

    this._emitListening = function (this: Listener) {
      // hopr-connect does not enable IPv6 connections right now, therefore we can set `listeningAddrs` statically
      // to `/ip4/0.0.0.0/tcp/0`, meaning listening on IPv4 using a canonical port
      // TODO check IPv6
      this.connectComponents.getAddressFilter().setAddrs(this.getAddrs(), [new Multiaddr(`/ip4/0.0.0.0/tcp/0`)])

      const usedRelays = this.connectComponents.getEntryNodes().getUsedRelayAddresses()

      if (usedRelays && usedRelays.length > 0) {
        const relayPeerIds = this.connectComponents
          .getEntryNodes()
          .getUsedRelayAddresses()
          .map((ma: Multiaddr) => {
            const tuples = ma.tuples()

            return peerIdFromBytes((tuples[0][1] as Uint8Array).slice(1))
          })

        this.connectComponents.getRelay().setUsedRelays(relayPeerIds)
      }

      this.dispatchEvent(new CustomEvent('listening'))
    }.bind(this)

    this.protocols = [
      {
        identifier: 'STUN server',
        isProtocol: isStun,
        takeStream: handleTcpStunRequest
      }
    ]
  }

  attachSocketHandlers() {
    this.udpSocket.once('close', () => {
      if (![ListenerState.CLOSING, ListenerState.CLOSED].includes(this.state)) {
        console.trace(`UDP socket was closed earlier than expected. Please report this!`)
      }
    })

    this.tcpSocket.once('close', () => {
      if (![ListenerState.CLOSING, ListenerState.CLOSED].includes(this.state)) {
        console.trace(`TCP socket was closed earlier than expected. Please report this!`)
      }
    })

    // Forward socket errors
    this.tcpSocket.on('error', (err) => this.dispatchEvent(new CustomEvent<Error>('error', { detail: err })))
    this.udpSocket.on('error', (err) => this.dispatchEvent(new CustomEvent<Error>('error', { detail: err })))

    this.tcpSocket.on('connection', async (socket: TCPSocket) => {
      try {
        await this.onTCPConnection(socket)
      } catch (err) {
        error(`network error`, err)
      }
    })
    this.udpSocket.on('message', (msg: Buffer, rinfo: RemoteInfo) => handleUdpStunRequest(this.udpSocket, msg, rinfo))
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

    if (this.components.getPeerId().toString() !== ma.getPeerId()) {
      let tmpListeningAddr = ma.decapsulateCode(CODE_P2P)

      if (!tmpListeningAddr.isThinWaistAddress()) {
        throw Error(`Unable to bind socket to <${tmpListeningAddr.toString()}>`)
      }

      // Replace wrong PeerId in given listeningAddr with own PeerId
      log(`replacing peerId in ${ma.toString()} by our peerId which is ${this.components.getPeerId().toString()}`)
      this.listeningAddr = tmpListeningAddr.encapsulate(`/p2p/${this.components.getPeerId().toString()}`)
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
      await this.listenTCP(options)
      await this.listenUDP(options.port)
    }
  }

  /**
   * Attaches the listener to TCP and UDP sockets
   * @param ma address to listen to
   */
  async listen(ma: Multiaddr): Promise<void> {
    if (this.state == ListenerState.CLOSED) {
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
        nodeToMultiaddr({
          address: natSituation.externalAddress,
          port: natSituation.externalPort,
          family: 'IPv4'
        })
      ]
    }

    this.addrs.interface = internalInterfaces.map(nodeToMultiaddr)

    this.attachSocketHandlers()

    // Need to be called before _emitListening
    // because _emitListening() sets an attribute in
    // the relay object
    this.connectComponents.getRelay().start()

    this._emitListening()

    // Only add relay nodes if node is not directly reachable or running locally
    if (this.testingOptions.__runningLocally || natSituation.bidirectionalNAT || !natSituation.isExposed) {
      this.connectComponents.getEntryNodes().on(RELAY_CHANGED_EVENT, this._emitListening)

      // Instructs entry node manager to assign to available
      // entry once startup has finished
      this.connectComponents.getEntryNodes().enable()
    }

    this.state = ListenerState.LISTENING
  }

  /**
   * Closes the listener and closes underlying TCP and UDP sockets.
   * @dev ignores prematurely closed TCP sockets
   */
  async close(): Promise<void> {
    this.state = ListenerState.CLOSING

    // Remove listeners
    this.connectComponents.getEntryNodes().stop()
    this.connectComponents.getEntryNodes().off(RELAY_CHANGED_EVENT, this._emitListening)

    this.stopUdpSocketKeepAliveInterval?.()

    await Promise.all([this.closeUDP(), this.closeTCP()])

    this.state = ListenerState.CLOSED
    this.connectComponents.getRelay().stop()
    this.dispatchEvent(new CustomEvent('close'))
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

    await Promise.all(this.__connections.map((conn: Connection) => attemptClose(conn, error)))

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
   * Used to determine which addresses to announce in the network.
   * @dev Should be called after `listen()` has returned
   * @dev List gets updated while waiting for `listen()`
   * @returns list of addresses under which the node is available
   */
  getAddrs(): Multiaddr[] {
    return [
      ...this.addrs.external,
      ...this.connectComponents.getEntryNodes().getUsedRelayAddresses(),
      ...this.addrs.interface
    ]
  }

  /**
   * Tracks connections to close them once necessary.
   * @param maConn connection to track
   */
  private trackConn(maConn: Connection) {
    this.__connections.push(maConn)
    verbose(`currently tracking ${this.__connections.length} connections ++`)

    return () => {
      verbose(`currently tracking ${this.__connections.length} connections --`)
      let index = this.__connections.findIndex((c: Connection) => c.id === maConn.id)

      if (index < 0) {
        // connection not found
        verbose(`DEBUG: Connection not found.`, maConn)
        return
      }

      if ([index + 1, 1].includes(this.__connections.length)) {
        this.__connections.pop()
      } else {
        this.__connections[index] = this.__connections.pop() as Connection
      }
    }
  }

  /**
   * Called on incoming TCP Connections. Initiates libp2p handshakes.
   * @param socket socket of incoming connection
   */
  private async onTCPConnection(socket: TCPSocket) {
    // Avoid uncaught errors caused by unstable connections
    socket.on('error', (err) => error('socket error', err))

    let maConn: MultiaddrConnection | undefined
    let conn: Connection | undefined

    try {
      maConn = fromSocket(socket, () => {
        if (conn) {
          this.components.getUpgrader().dispatchEvent(
            new CustomEvent(`connectionEnd`, {
              detail: conn
            })
          )
        }
      }) as any
    } catch (err: any) {
      error(`inbound connection failed. ${err.message}`)
    }

    if (maConn == undefined) {
      socket.destroy()
      return
    }

    const it = (maConn.source as AsyncIterable<Uint8Array>)[Symbol.asyncIterator]()
    const firstMessage = await it.next()

    const stream = (async function* () {
      yield firstMessage.value

      yield* it as any
    })() as any

    for (const additionalProtocol of this.protocols) {
      if (additionalProtocol.isProtocol(firstMessage.value)) {
        additionalProtocol.takeStream(socket, stream)
        return
      }
    }

    maConn.source = stream

    log('new inbound connection %s', maConn.remoteAddr)

    try {
      conn = await this.components.getUpgrader().upgradeInbound(maConn)
    } catch (err: any) {
      if (!err) {
        error('inbound connection failed. empty error')
      } else {
        switch (err.code) {
          case 'ERR_CONNECTION_INTERCEPTED':
            error(`inbound connection failed. Node is not registered.`)
            break
          case 'ERR_ENCRYPTION_FAILED':
            error(`inbound connection failed because encryption failed. Maybe connected to the wrong node?`)
            break
          default:
            error('inbound connection failed', err)
        }
      }

      if (maConn != undefined) {
        return attemptClose(maConn, error)
      }

      return
    }

    cleanExistingConnections(this.components, conn.remotePeer, conn.id, error)

    if (conn.tags) {
      conn.tags.push(PeerConnectionType.DIRECT)
    } else {
      conn.tags = [PeerConnectionType.DIRECT]
    }

    log('inbound connection %s upgraded', maConn.remoteAddr)

    socket.once('close', this.trackConn(conn))
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
   * Tries to determine a node's public IP address by
   * using STUN servers
   * @param ownAddress the host on which we are listening
   * @param ownPort the port on which we are listening
   * @returns Promise that resolves once STUN request came back or STUN timeout was reched
   */
  async checkNATSituation(ownAddress: string, ownPort: number): Promise<NATSituation> {
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

    const usableStunServers = this.getUsableStunServers(ownAddress, ownPort)

    // allocate UDP port mapping
    let externalInterface = await getExternalIp(
      usableStunServers,
      this.udpSocket,
      this.testingOptions.__preferLocalAddresses
    )

    if (externalInterface == undefined) {
      return {
        bidirectionalNAT: true
      }
    }

    this.stopUdpSocketKeepAliveInterval = retimer(
      async () => {
        const multiaddrs = this.getUsableStunServers(ownAddress, ownPort)

        log(`Re-allocating NAT UDP mapping`, multiaddrs)

        await getExternalIp(multiaddrs, this.udpSocket, this.testingOptions.__preferLocalAddresses)
      },
      // Following recommendations of https://www.rfc-editor.org/rfc/rfc5626
      () => randomInteger(24_000, 29_000)
    )

    const isUdpExposed = await isExposedHost(usableStunServers, this.tcpSocket, this.udpSocket, externalInterface.port)

    return {
      bidirectionalNAT: false,
      externalAddress: externalInterface.address,
      externalPort: externalInterface.port,
      // assuming that TCP is exposed as well
      // @TODO add TCP STUN support
      isExposed: isUdpExposed
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

    let usableNodes: PeerStoreType[] = this.connectComponents.getEntryNodes().getAvailabeEntryNodes()

    if (usableNodes.length == 0) {
      // Use unchecked nodes at startup
      usableNodes = this.connectComponents.getEntryNodes().getUncheckedEntryNodes()
    }

    for (const usableNode of usableNodes) {
      if (usableNode.id.equals(this.components.getPeerId())) {
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
