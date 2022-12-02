import { type Socket } from 'dgram'
import { type Server } from 'net'

import { Multiaddr } from '@multiformats/multiaddr'
import debug from 'debug'

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

/**
 * Tries to determine the external IPv4 address using STUN over UDP
 * @returns Addr+Port or undefined if the STUN response are ambiguous (e.g. bidirectional NAT)
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
        if (multiAddrs != undefined && multiAddrs.length > 0) {
          yield* randomIterator(multiAddrs)
        }
        // Fallback option
        yield* PUBLIC_UDP_STUN_SERVERS
      })(),
      socket,
      undefined
    )
  )[0]
}

/**
 *
 * @param multiAddrs list of STUN servers
 * @param tcpServer tcp server to use for outgoing
 * @param udpSocket
 * @param port
 * @param runningLocally
 * @returns
 */
export async function isExposedHost(
  multiAddrs: Multiaddr[],
  tcpServer: Server,
  udpSocket: Socket,
  port: number,
  runningLocally = false
): Promise<boolean> {
  // Performs a STUN request from the given socket and thereby creates
  // a mapping in the DHT. In some cases, this is sufficient to also
  // receive TCP packets on that port.
  const udpMapped = await isUdpExposedHost(
    (function* () {
      if (multiAddrs != undefined && multiAddrs.length > 0) {
        yield* randomIterator(multiAddrs)
      }
      yield* PUBLIC_UDP_RFC_5780_SERVERS
    })(),
    udpSocket,
    undefined,
    port,
    runningLocally
  )

  const tcpMapped = await isTcpExposedHost(
    (function* () {
      if (multiAddrs != undefined && multiAddrs.length > 0) {
        yield* randomIterator(multiAddrs)
      }
      yield* PUBLIC_TCP_RFC_5780_SERVERS
    })(),
    tcpServer,
    undefined,
    port,
    runningLocally
  )

  log(`NAT measurement: TCP ${exposedResponseToString(tcpMapped)}, UDP ${exposedResponseToString(udpMapped)}`)

  switch (tcpMapped) {
    case STUN_EXPOSED_CHECK_RESPOSE.EXPOSED:
      switch (udpMapped) {
        case STUN_EXPOSED_CHECK_RESPOSE.EXPOSED:
        case STUN_EXPOSED_CHECK_RESPOSE.UNKNOWN:
          return true
        default:
          return false
      }
    case STUN_EXPOSED_CHECK_RESPOSE.UNKNOWN:
      switch (udpMapped) {
        case STUN_EXPOSED_CHECK_RESPOSE.EXPOSED:
          return true
        default:
          return false
      }
    case STUN_EXPOSED_CHECK_RESPOSE.NOT_EXPOSED:
      return false
  }
}

export { handleTcpStunRequest, PUBLIC_RFC_5780_SERVERS } from './tcp/index.js'
export { handleUdpStunRequest, PUBLIC_UDP_RFC_5780_SERVERS } from './udp/index.js'
