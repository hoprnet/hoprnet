import { type Socket } from 'dgram'
import { type Socket as TCPSocket } from 'net'

import { Multiaddr } from '@multiformats/multiaddr'
import debug from 'debug'

import { create_gauge } from '@hoprnet/hopr-utils'

import { randomIterator } from '../../utils/index.js'
import { isUdpExposedHost, performSTUNRequests } from './udp/index.js'

import { PUBLIC_UDP_RFC_5780_SERVERS, PUBLIC_UDP_STUN_SERVERS } from './udp/constants.js'
import { isTcpExposedHost } from './tcp/index.js'
import { exposedResponseToString, STUN_EXPOSED_CHECK_RESPOSE } from './constants.js'
import { PUBLIC_TCP_RFC_5780_SERVERS } from './tcp/constants.js'

type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

const log = debug('hopr-connect:stun')

const metric_isExposed = create_gauge(
  `connect_node_is_exposed`,
  `Shows whether a node believes to run on an exposed host`
)

/**
 * Tries to determine the external IPv4 address using STUN over UDP
 * @returns Addr+Port or undefined if the STUN responses are ambiguous (e.g. due to bidirectional NAT)
 *
 * @param multiAddrs Multiaddrs to use as STUN servers
 * @param socket Node.JS socket to use for the STUN request
 * @param __preferLocalAddress [testing] assume that all nodes run in a local network
 */
export async function getExternalIp(
  multiAddrs: Multiaddr[] | undefined,
  socket: Socket,
  __preferLocalAddress = false
): Promise<Interface | undefined> {
  if (__preferLocalAddress) {
    if (multiAddrs == undefined || multiAddrs.length == 0) {
      const socketAddress = socket.address() as Interface | null
      if (socketAddress == null) {
        throw Error(`Socket is not listening`)
      }

      log(
        `Running in local-mode without any given STUN server, assuming that socket address 127.0.0.1:${socketAddress.port} is public address`
      )
      return {
        ...socketAddress,
        address: '127.0.0.1',
        family: 'IPv4'
      }
    }

    return (await performSTUNRequests(randomIterator(multiAddrs), socket, undefined, true))[0]
  }

  return (
    await performSTUNRequests(
      (function* () {
        // Intermediate solution, to be changed once more nodes are upgraded
        // Fallback option
        if (!__preferLocalAddress) {
          yield* PUBLIC_UDP_STUN_SERVERS
        }
        if (multiAddrs != undefined && multiAddrs.length > 0) {
          yield* randomIterator(multiAddrs)
        }
      })(),
      socket,
      undefined
    )
  )[0]
}

/**
 * Checks whether a given port is publicly accessible using TCP *and* UDP.
 *
 * Used to determine whether a node can act as a public relay node.
 *
 * @param multiAddrs list of STUN servers
 * @param addTcpProtocolListener adds a connection listener to an *existing* TCP socket
 * @param udpSocket UDP socket to send and receive messages
 * @param port port to be checked
 * @param runningLocally [testing] local-mode STUN, used for unit and e2e testing
 * @returns
 */
export async function isExposedHost(
  multiAddrs: Multiaddr[],
  addTcpProtocolListener: (listener: (socket: TCPSocket, stream: AsyncIterable<Uint8Array>) => void) => () => void,
  udpSocket: Socket,
  port: number,
  runningLocally = false
): Promise<boolean> {
  // Performs a STUN request from the given socket and thereby creates
  // a mapping in the NAT table. In some cases, this is sufficient to also
  // receive TCP packets on that port.
  const udpMapped = await isUdpExposedHost(
    (function* () {
      // Intermediate solution, to be changed once more nodes are upgraded
      if (!runningLocally) {
        yield* PUBLIC_UDP_RFC_5780_SERVERS
      }

      if (multiAddrs != undefined && multiAddrs.length > 0) {
        yield* randomIterator(multiAddrs)
      }
    })(),
    udpSocket,
    undefined,
    port,
    runningLocally
  )

  const tcpMapped = await isTcpExposedHost(
    (function* () {
      // Intermediate solution, to be changed once more nodes are upgraded
      if (!runningLocally) {
        yield* PUBLIC_TCP_RFC_5780_SERVERS
      }

      if (multiAddrs != undefined && multiAddrs.length > 0) {
        yield* randomIterator(multiAddrs)
      }
    })(),
    addTcpProtocolListener,
    undefined,
    port
  )

  log(
    `NAT measurement: TCP socket ${exposedResponseToString(tcpMapped)}, UDP socket ${exposedResponseToString(
      udpMapped
    )}`
  )

  let isExposed: boolean

  // Intermediate solution. To be hardened once more nodes are upgraded
  switch (tcpMapped) {
    case STUN_EXPOSED_CHECK_RESPOSE.EXPOSED:
      switch (udpMapped) {
        case STUN_EXPOSED_CHECK_RESPOSE.EXPOSED:
        case STUN_EXPOSED_CHECK_RESPOSE.UNKNOWN:
          isExposed = true
          break
        default:
          isExposed = false
          break
      }
      break
    case STUN_EXPOSED_CHECK_RESPOSE.UNKNOWN:
      switch (udpMapped) {
        case STUN_EXPOSED_CHECK_RESPOSE.EXPOSED:
          isExposed = true
          break
        default:
          isExposed = false
          break
      }
      break
    case STUN_EXPOSED_CHECK_RESPOSE.NOT_EXPOSED:
      isExposed = false
      break
  }

  metric_isExposed.set(isExposed ? 1 : 0)

  return isExposed
}

export { handleTcpStunRequest, PUBLIC_RFC_5780_SERVERS } from './tcp/index.js'
export { handleUdpStunRequest, PUBLIC_UDP_RFC_5780_SERVERS } from './udp/index.js'
