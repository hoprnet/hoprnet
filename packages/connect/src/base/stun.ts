// @ts-ignore untyped module
import { decode, constants, createMessage, createTransaction, validateFingerprint } from 'stun'

// @ts-ignore untyped module
import isStun from 'is-stun'

import type { Socket, RemoteInfo } from 'dgram'
import { Multiaddr } from '@multiformats/multiaddr'
import debug from 'debug'
import {
  randomSubset,
  ipToU8aAddress,
  isLocalhost,
  isPrivateAddress,
  u8aAddrToString,
  u8aToNumber
} from '@hoprnet/hopr-utils'
import { CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6 } from '../constants.js'
// @ts-ignore untyped module
import retimer from 'retimer'

const log = debug('hopr-connect:stun:error')
const error = debug('hopr-connect:stun:error')
const verbose = debug('hopr-connect:verbose:stun')

export type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

export const STUN_TIMEOUT = 1000

// Only used to determine the external address of the bootstrap server
export const PUBLIC_STUN_SERVERS = [
  new Multiaddr(`/dns4/stun.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun1.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun2.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun3.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun4.l.google.com/udp/19302`),
  new Multiaddr(`/dns4/stun.sipgate.net/udp/3478`),
  new Multiaddr(`/dns4/stun.callwithus.com/udp/3478`)
]

export const DEFAULT_PARALLEL_STUN_CALLS = 4

// STUN server constants
const isStunRequest = 0x0000
// const isStunIndication = 0x0010
const isStunSuccessResponse = 0x0100
// const isStunErrorResponse = 0x0110
const kStunTypeMask = 0x0110

/**
 * Handles STUN requests
 * @param socket Node.JS socket to use
 * @param data received packet
 * @param rinfo Addr+Port of the incoming connection
 * @param __fakeRInfo [testing] overwrite incoming information to intentionally send misleading STUN response
 */
export function handleStunRequest(socket: Socket, data: Buffer, rinfo: RemoteInfo, __fakeRInfo?: RemoteInfo): void {
  let replyAddress = rinfo.address

  // When using 'udp6' sockets, IPv4 addresses get prefixed by ::ffff:
  if (rinfo.family === 'IPv6') {
    const match = rinfo.address.match(/(?<=::ffff:)[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/)

    if (match) {
      rinfo.family = 'IPv4'
      rinfo.address = match[0]
    }
  }

  if (!isStun(data)) {
    return
  }

  const stunMessage = decode(data)

  switch (stunMessage.type & kStunTypeMask) {
    case isStunRequest:
      const message = createMessage(constants.STUN_BINDING_RESPONSE, stunMessage.transactionId)

      verbose(`Received ${stunMessage.isLegacy() ? 'legacy ' : ''}STUN request from ${rinfo.address}:${rinfo.port}`)

      let addrInfo = rinfo
      if (__fakeRInfo) {
        if (__fakeRInfo.family === 'IPv6') {
          const match = __fakeRInfo.address.match(/(?<=::ffff:)[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}/)

          if (match) {
            addrInfo = {
              ...rinfo,
              family: 'IPv4',
              port: __fakeRInfo.port,
              address: match[0]
            }
          } else {
            addrInfo = __fakeRInfo
          }
        } else {
          addrInfo = __fakeRInfo
        }
      }

      // To be compliant with RFC 3489
      if (stunMessage.isLegacy()) {
        // Copy magic STUN cookie as specified by RFC 5389
        message[Symbol.for('kCookie')] = stunMessage[Symbol.for('kCookie')]
        message.addAttribute(constants.STUN_ATTR_MAPPED_ADDRESS, addrInfo.address, addrInfo.port)
        socket.send(message.toBuffer(), rinfo.port, replyAddress)
        return
      }

      // Comply with RFC 5780
      message.addAttribute(constants.STUN_ATTR_XOR_MAPPED_ADDRESS, addrInfo.address, addrInfo.port)
      message.addFingerprint()

      socket.send(message.toBuffer(), rinfo.port, replyAddress)

      break
    default:
      break
  }
}

export type Request = {
  multiaddr: Multiaddr
  tId: Buffer
  response?: Interface
}

type RequestWithResponse = Required<Request>

function isRequestWithResponse(request: Request): request is RequestWithResponse {
  if (request.response == undefined) {
    return false
  }
  return true
}

/**
 * Takes a list of STUN servers and tries them one-by-one
 * in random order.
 * @param multiaddrs list of STUN servers
 * @param socket socket to receive replies
 * @param maxAttempts [optional] maximum number of attempts
 * @param runningLocally [optional] enable STUN local-mode
 * @returns STUN responses
 */
export async function iterateThroughStunServers(
  multiaddrs: Multiaddr[],
  socket: Socket,
  maxAttempts = Infinity,
  runningLocally = false
): Promise<RequestWithResponse[]> {
  const usedStunServers = new Set<string>()

  let selectedStunServers: Multiaddr[]
  if (multiaddrs.length > DEFAULT_PARALLEL_STUN_CALLS) {
    selectedStunServers = randomSubset(multiaddrs, DEFAULT_PARALLEL_STUN_CALLS)
  } else {
    selectedStunServers = multiaddrs
  }

  let responses: RequestWithResponse[] = await performSTUNRequests(
    selectedStunServers,
    socket,
    STUN_TIMEOUT,
    runningLocally
  )

  if (multiaddrs.length > DEFAULT_PARALLEL_STUN_CALLS) {
    while (responses.length < 2) {
      for (const selected of selectedStunServers) {
        usedStunServers.add(selected.toString())
      }

      if (usedStunServers.size >= maxAttempts) {
        break
      }

      const toFetch = Math.min(maxAttempts, DEFAULT_PARALLEL_STUN_CALLS + usedStunServers.size) - usedStunServers.size

      selectedStunServers = randomSubset(multiaddrs, toFetch, (ma: Multiaddr) => !usedStunServers.has(ma.toString()))

      responses.push(...(await performSTUNRequests(selectedStunServers, socket, STUN_TIMEOUT, runningLocally)))

      if (selectedStunServers.length < toFetch) {
        break
      }
    }
  }

  return responses
}
/**
 * Performs STUN requests and returns their responses, if any
 * @param multiAddrs STUN servers to contact
 * @param socket socket to send requests and receive responses
 * @param timeout STUN timeout
 * @param runningLocally [optional] enable STUN local-mode
 * @returns the responses, if any
 */
export async function performSTUNRequests(
  multiAddrs: Multiaddr[],
  socket: Socket,
  timeout = STUN_TIMEOUT,
  runningLocally = false
): Promise<RequestWithResponse[]> {
  const requests: Request[] = []
  for (const multiaddr of multiAddrs) {
    requests.push({
      multiaddr,
      tId: createTransaction()
    })
  }
  // Assign the event handler before sending the requests
  const results = decodeIncomingSTUNResponses(requests, socket, timeout)
  // Everything is set up, so we can dispatch the requests
  sendStunRequests(requests, socket)

  const responses = await results

  return getUsableResults(responses ?? [], runningLocally)
}

/**
 * Send requests to given STUN servers
 * @param addrs requests with addr and transaction id
 * @param socket the socket to send the STUN requests
 * @returns usable transaction IDs and the corresponding multiaddrs
 */
function sendStunRequests(addrs: Request[], socket: Socket): void {
  for (const addr of addrs) {
    if (![CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6].includes(addr.multiaddr.tuples()[0][0])) {
      error(`Cannot contact STUN server ${addr.multiaddr.toString()} due to invalid address.`)
      continue
    }

    const tuples = addr.multiaddr.tuples()

    if (tuples.length == 0) {
      throw Error(`Cannot perform STUN request: empty Multiaddr`)
    }

    let address: string

    switch (tuples[0][0]) {
      case CODE_DNS4:
      case CODE_DNS6:
        address = new TextDecoder().decode(tuples[0][1] as Uint8Array)
        break
      case CODE_IP6:
        address = u8aAddrToString(tuples[0][1] as Uint8Array, 'IPv6')
        break
      case CODE_IP4:
        address = `::ffff:${u8aAddrToString(tuples[0][1] as Uint8Array, 'IPv4')}`
        break
      default:
        throw Error(`Invalid address: ${addr.multiaddr.toString()}`)
    }

    const port: number | undefined = tuples.length >= 2 ? u8aToNumber(tuples[1][1] as Uint8Array) : undefined

    const message = createMessage(constants.STUN_BINDING_REQUEST, addr.tId)

    message.addFingerprint()

    socket.send(message.toBuffer(), port, address, (err?: any) => {
      if (err) {
        error(err.message)
      } else {
        verbose(`STUN request successfully sent to ${address}:${port}`)
      }
    })
  }
}

function decodeIncomingSTUNResponses(addrs: Request[], socket: Socket, ms: number = STUN_TIMEOUT): Promise<Request[]> {
  return new Promise<Request[]>((resolve) => {
    let responsesReceived = 0
    let finished = false

    const listener = (data: Buffer) => {
      if (!isStun(data)) {
        return
      }

      const response = decode(data)

      // Don't check for STUN FINGERPRINT since external
      // STUN servers might not support this feature

      switch (response.type & kStunTypeMask) {
        case isStunSuccessResponse:
          for (const addr of addrs) {
            if (Buffer.compare(addr.tId, response.transactionId) === 0) {
              if (addr.response != undefined) {
                continue
              }

              addr.response = response.getXorAddress() ?? response.getAddress()
              // If we have filled an entry, increase response counter
              if (addr.response != undefined) {
                responsesReceived++
              }
              break
            }
          }

          if (responsesReceived == addrs.length) {
            log(`STUN success. ${responsesReceived} of ${addrs.length} servers replied.`)
            done()
          }
          break
        default:
          break
      }
    }

    const done = () => {
      if (finished) {
        return
      }
      finished = true

      socket.removeListener('message', listener)
      timer.clear()

      resolve(addrs)
    }

    const timer = retimer(() => {
      log(
        `STUN timeout. ${addrs.filter((addr) => addr.response).length} of ${
          addrs.length
        } selected STUN servers replied.`
      )
      done()
    }, ms)

    // Receiving a Buffer, not a Uint8Array
    socket.on('message', listener)
  })
}

/**
 * Remove unusable responses from results
 * @param responses results to filter
 * @param runningLocally whether to run in local-mode or not
 * @returns filtered results
 */
export function getUsableResults(responses: Request[], runningLocally = false): RequestWithResponse[] {
  let filtered: RequestWithResponse[] = []

  for (const result of responses) {
    if (!isRequestWithResponse(result)) {
      continue
    }

    switch (result.response.family) {
      case 'IPv6':
        // STUN over IPv6 is not yet supported
        break
      case 'IPv4':
        const u8aAddr = ipToU8aAddress(result.response.address, 'IPv4')

        if (runningLocally) {
          // Only take local or private addresses
          if (isPrivateAddress(u8aAddr, 'IPv4') || isLocalhost(u8aAddr, 'IPv4')) {
            filtered.push(result)
          }
        } else {
          // Only take public addresses
          if (!isPrivateAddress(u8aAddr, 'IPv4') && !isLocalhost(u8aAddr, 'IPv4')) {
            filtered.push(result)
          }
        }
        break
    }
  }

  return filtered
}

/**
 * Check if the results are ambiguous and return single response
 * if not ambiguous.
 * @param results results to check for ambiguity
 * @returns a public address if, and only if, the results are not ambiguous
 */
export function intepreteResults(results: RequestWithResponse[]):
  | {
      ambiguous: true
    }
  | {
      ambiguous: false
      publicAddress: Interface
    } {
  if (results.length == 0 || results[0].response == undefined) {
    return { ambiguous: true }
  }

  for (const [index, result] of results.entries()) {
    if (index == 0) {
      continue
    }

    if (result.response.address != results[0].response.address || result.response.port != results[0].response.port) {
      return { ambiguous: true }
    }
  }

  return {
    ambiguous: false,
    publicAddress: results[0].response
  }
}

/**
 * Tries to determine the external IPv4 address
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
  let responses: RequestWithResponse[] = []
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

    responses.push(...(await iterateThroughStunServers(multiAddrs, socket, Infinity, true)))

    if (responses.length == 0) {
      log(`Cannot determine external IP because running in local mode and none of the local STUN servers replied.`)
      return
    }
  } else {
    if (multiAddrs == undefined || multiAddrs.length == 0) {
      responses.push(...(await iterateThroughStunServers(PUBLIC_STUN_SERVERS, socket)))
    } else {
      responses.push(...(await iterateThroughStunServers(multiAddrs, socket)))

      // We need at least two answers to determine whether the node
      // operates behind a bidirectional NAT
      if (responses.length < 2) {
        responses.push(...(await iterateThroughStunServers(PUBLIC_STUN_SERVERS, socket)))
      }
    }
  }

  if (responses.length < 2) {
    // We have tried all ways to check if the node
    // is operating behind a bidirectional NAT but we
    // could not get more than one response from STUN servers
    log(
      `Could not get more than one response from available STUN servers. Assuming that node operates behind a bidirectional NAT`
    )
    return
  }

  const interpreted = intepreteResults(responses)

  if (interpreted.ambiguous) {
    log(`Received STUN results are ambiguous. Assuming that node operates behind a bidirectional NAT`)
    return
  } else {
    return interpreted.publicAddress
  }
}
