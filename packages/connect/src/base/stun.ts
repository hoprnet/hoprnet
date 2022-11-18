// @ts-ignore untyped module
import { decode, constants, createMessage, createTransaction, validateFingerprint } from 'stun'

// @ts-ignore untyped module
import isStun from 'is-stun'

import { Socket, RemoteInfo, createSocket } from 'dgram'
import { Multiaddr } from '@multiformats/multiaddr'
import debug from 'debug'
import {
  ipToU8aAddress,
  isLocalhost,
  isPrivateAddress,
  u8aAddrToString,
  u8aToNumber,
  u8aToHex,
  randomPermutation
} from '@hoprnet/hopr-utils'
import { CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6 } from '../constants.js'

// @ts-ignore untyped module
import retimer from 'retimer'
import { upd6Lookup } from '../utils/index.js'

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

  const request = decode(data)

  switch (request.type & kStunTypeMask) {
    case isStunRequest:
      const response = createMessage(constants.STUN_BINDING_RESPONSE, request.transactionId)

      verbose(`Received ${request.isLegacy() ? 'legacy ' : ''}STUN request from ${rinfo.address}:${rinfo.port}`)

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
      if (request.isLegacy()) {
        // Copy magic STUN cookie as specified by RFC 5389
        response[Symbol.for('kCookie')] = request[Symbol.for('kCookie')]
        response.addAttribute(constants.STUN_ATTR_MAPPED_ADDRESS, addrInfo.address, addrInfo.port)
        socket.send(response.toBuffer(), rinfo.port, replyAddress)
        return
      }

      let replyPort = addrInfo.port

      // RESPONSE_PORT can be 0
      const responsePort = request.getAttribute(constants.STUN_ATTR_RESPONSE_PORT)
      if (responsePort != undefined) {
        replyPort = responsePort.value
      }

      // Comply with RFC 5780
      response.addAttribute(constants.STUN_ATTR_MAPPED_ADDRESS, addrInfo.address, addrInfo.port)
      response.addAttribute(constants.STUN_ATTR_XOR_MAPPED_ADDRESS, addrInfo.address, addrInfo.port)

      // Allows multiplexing STUN protocol with other protocols
      response.addFingerprint()

      socket.send(response.toBuffer(), replyPort, replyAddress)

      break
    default:
      break
  }
}

export type Request = {
  multiaddr: Multiaddr
  responsePort?: number
  response?: Interface
  timeout: any
}

type Requests = Map<string, Request>

enum STUN_ALIVE_STATE {
  SEARCHING_STUN_SERVER,
  SEARCHING_RFC_5780_STUN_SERVER,
  CHECKING_PORT_MAPPING
}

export function isBindingAlive(
  multiaddrs: Iterable<Multiaddr>,
  socket: Socket,
  timeout = STUN_TIMEOUT,
  stunPort = socket.address().port,
  runningLocally = false
): Promise<boolean> {
  if (runningLocally) {
    return Promise.resolve(true)
  }

  return new Promise<boolean>(async (resolve, reject) => {
    const requests = new Map<string, Request & { state: STUN_ALIVE_STATE }>()

    const secondarySocket = createSocket({
      type: 'udp6',
      lookup: upd6Lookup
    })

    const secondaryInterface = await performSTUNRequests(multiaddrs, secondarySocket, timeout, runningLocally)

    if (secondaryInterface == undefined) {
      // Endpoint-dependent mapping, most likely bidirectional NAT
      resolve(false)
      return
    }

    let stopListening: () => void
    let stopListeningSecondary: () => void

    const end = () => {
      log(`ending`)
      stopListening()
      stopListeningSecondary()
      secondarySocket.close()
    }

    const it = multiaddrs[Symbol.iterator]()

    const onTimeoutSecondary = (transactionId: Buffer) => {
      requests.delete(u8aToHex(transactionId))

      nextSTUNRequest(
        it,
        requests,
        timeout,
        secondarySocket,
        secondaryInterface.port,
        onTimeoutSecondary,
        onError,
        STUN_ALIVE_STATE.SEARCHING_RFC_5780_STUN_SERVER
      )
    }

    const onTimeoutPrimary = (transactionId: Buffer) => {
      const tIdString = u8aToHex(transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(`Received unexpected STUN response from. Dropping response`)
        return
      }

      log(`onTimeout primary`, tIdString)

      end()
      resolve(false)
      return
    }

    const updateSecondary = (response: { response: Interface; transactionId: Buffer }) => {
      const tIdString = u8aToHex(response.transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(
          `Received unexpected STUN response from ${response.response.address}:${response.response.port}. Dropping response`
        )
        return
      }

      log(`update secondary`, request.state, tIdString)

      request.timeout.clear()

      requests.delete(tIdString)

      switch (request.state) {
        case STUN_ALIVE_STATE.SEARCHING_RFC_5780_STUN_SERVER:
          nextSTUNRequest(
            [request.multiaddr][Symbol.iterator](),
            requests,
            timeout,
            secondarySocket,
            stunPort,
            onTimeoutPrimary,
            () => {},
            STUN_ALIVE_STATE.CHECKING_PORT_MAPPING
          )
          break
        case STUN_ALIVE_STATE.CHECKING_PORT_MAPPING:
          // STUN server does not understand RESPONSE_PORT extension
          log(`not useful, move to next`)
          nextSTUNRequest(
            it,
            requests,
            timeout,
            secondarySocket,
            secondaryInterface.port,
            onTimeoutSecondary,
            () => {},
            STUN_ALIVE_STATE.SEARCHING_RFC_5780_STUN_SERVER
          )
          break
      }
    }

    const updatePrimary = (response: { response: Interface; transactionId: Buffer }) => {
      const tIdString = u8aToHex(response.transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(
          `Received unexpected STUN response from ${response.response.address}:${response.response.port}. Dropping response`
        )
        return
      }

      log(`Update primary`, request.state, tIdString)

      request.timeout.clear()
      requests.delete(tIdString)

      switch (request.state) {
        case STUN_ALIVE_STATE.CHECKING_PORT_MAPPING:
          end()
          resolve(true)
          break
        default:
          // Unexpected request, trying another STUN server
          nextSTUNRequest(
            it,
            requests,
            timeout,
            secondarySocket,
            secondaryInterface.port,
            onTimeoutSecondary,
            () => {},
            STUN_ALIVE_STATE.CHECKING_PORT_MAPPING
          )
          break
      }
    }

    stopListening = decodeIncomingSTUNResponses(socket, updatePrimary)
    stopListeningSecondary = decodeIncomingSTUNResponses(secondarySocket, updateSecondary)

    const onError = (err: any) => {
      end()
      reject(err)
    }

    log(`first measurement`)
    nextSTUNRequest(
      it,
      requests,
      timeout,
      secondarySocket,
      secondaryInterface.port,
      onTimeoutSecondary,
      onError,
      STUN_ALIVE_STATE.SEARCHING_RFC_5780_STUN_SERVER
    )
  })
}

function sameEndpoint(first: Interface, second: Interface): boolean {
  return first.address === second.address && first.port == second.port
}

function nextSTUNRequest(
  it: Iterator<Multiaddr>,
  requests: Map<string, Request & { state?: STUN_ALIVE_STATE }>,
  timeout: number,
  socket: Socket,
  stunPort: number | undefined,
  onTimeout: (tId: Buffer) => void,
  onError: (err: any) => void,
  state?: STUN_ALIVE_STATE
) {
  const chunk = it.next()

  if (chunk.done) {
    onError(Error(`Not enough STUN servers given to determine own public IP address`))
    return
  }

  const nextSTUNServer = {
    transactionId: createTransaction(),
    multiaddr: chunk.value
  }
  requests.set(u8aToHex(nextSTUNServer.transactionId), {
    multiaddr: nextSTUNServer.multiaddr,
    timeout: retimer(onTimeout, timeout, nextSTUNServer.transactionId),
    state
  })
  sendStunRequests(nextSTUNServer.multiaddr, nextSTUNServer.transactionId, stunPort, socket)

  return nextSTUNServer.transactionId
}

function sameResponse(requests: Map<string, Request>, response: { response: Interface; transactionId: Buffer }) {
  for (const [tid, storedRequest] of requests) {
    if (tid === u8aToHex(response.transactionId)) {
      continue
    }

    if (storedRequest.response != undefined) {
      return sameEndpoint(response.response, storedRequest.response) ? storedRequest.response : undefined
    }
  }
}
/**
 * Performs STUN requests and returns their responses, if any
 * @param multiaddrs STUN servers to contact
 * @param socket socket to send requests and receive responses
 * @param timeout STUN timeout
 * @param runningLocally [optional] enable STUN local-mode
 * @returns the responses, if any
 */
export function performSTUNRequests(
  multiaddrs: Iterable<Multiaddr>,
  socket: Socket,
  timeout = STUN_TIMEOUT,
  runningLocally = false
): Promise<Interface | undefined> {
  return new Promise<Interface | undefined>((resolve, reject) => {
    let successfulResponses = 0
    const requests: Requests = new Map<string, Request>()

    const it = multiaddrs[Symbol.iterator]()

    let stopListening: () => void

    let onTimeout = (transactionId: Buffer) => {
      requests.delete(u8aToHex(transactionId))
      nextSTUNRequest(it, requests, timeout, socket, undefined, onTimeout, onError)
    }

    const update = (response: { response: Interface; transactionId: Buffer }) => {
      const tIdString = u8aToHex(response.transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(
          `Received unexpected STUN response from ${response.response.address}:${response.response.port}. Dropping response`
        )
        return
      }

      request.timeout.clear()

      if (!isUsableResult(response.response, runningLocally)) {
        requests.delete(tIdString)

        nextSTUNRequest(it, requests, timeout, socket, undefined, onTimeout, onError)
        return
      }

      request.response = response.response
      requests.set(tIdString, request)

      successfulResponses++
      if (successfulResponses == 2) {
        stopListening()
        resolve(sameResponse(requests, response) != undefined ? response.response : undefined)
        return
      }
    }

    stopListening = decodeIncomingSTUNResponses(socket, update)

    const onError = (err: any) => {
      stopListening()
      reject(err)
    }

    for (let i = 0; i < 2; i++) {
      nextSTUNRequest(it, requests, timeout, socket, undefined, onTimeout, onError)
    }
  })
}

/**
 * Encodes STUN message and sends them using the given socket
 * to a STUN server.
 * @param multiaddr address to contact
 * @param tId
 * @param socket socket to send the STUN requests
 */
function sendStunRequests(multiaddr: Multiaddr, tId: Buffer, responsePort: number | undefined, socket: Socket): void {
  const tuples = multiaddr.tuples()

  if (tuples.length == 0) {
    throw Error(`Cannot perform STUN request: empty Multiaddr`)
  }

  let address: string

  switch (tuples[0][0]) {
    case CODE_DNS4:
    case CODE_DNS6:
      address = new TextDecoder().decode(tuples[0][1]?.slice(1) as Uint8Array)
      break
    case CODE_IP6:
      address = u8aAddrToString(tuples[0][1] as Uint8Array, 'IPv6')
      break
    case CODE_IP4:
      address = `::ffff:${u8aAddrToString(tuples[0][1] as Uint8Array, 'IPv4')}`
      break
    default:
      throw Error(`Invalid address: ${multiaddr.toString()}`)
  }

  const port: number | undefined = tuples.length >= 2 ? u8aToNumber(tuples[1][1] as Uint8Array) : undefined

  const message = createMessage(constants.STUN_BINDING_REQUEST, tId)

  // Response port can be 0
  if (responsePort != undefined) {
    message.addAttribute(constants.STUN_ATTR_RESPONSE_PORT, responsePort)
  }

  // Allows multiplexing of STUN protocol with other protocols
  // message.addFingerprint()

  socket.send(message.toBuffer(), port, address, (err?: any) => {
    if (err) {
      error(err.message)
    } else {
      verbose(
        `STUN request successfully sent to ${address}:${port} Transaction: ${u8aToHex(tId)}${
          responsePort != undefined ? ` port ${responsePort}` : ''
        }`
      )
    }
  })
}

/**
 * Attaches a listener to the given socket that calls the `update`
 * function on every reception of a BindingResponse
 * @param socket socket to listen for messages
 * @param update called on incoming STUN BindingResponses
 * @returns
 */
function decodeIncomingSTUNResponses(
  socket: Socket,
  update: (response: { response: Interface; transactionId: Buffer }) => void
): () => void {
  const listener = (data: Buffer, rinfo: any) => {
    if (!isStun(data)) {
      return
    }

    const response = decode(data)

    switch (response.type & kStunTypeMask) {
      case isStunSuccessResponse:
        update({
          response: response.getXorAddress() ?? response.getAddress(),
          transactionId: response.transactionId
        })
        break
      case isStunRequest:
        // handled by STUN server, ignoring
        break
      default:
        log(`unknown STUN response`, data, rinfo)
        break
    }
  }

  // Node.js sockets emit Buffers
  socket.on('message', listener)

  return () => socket.removeListener('message', listener)
}

/**
 * Filters STUN responses according to network situation, e.g. local testnet
 * @dev Drops IPv6 responses because IPv6 is not yet supported
 * IPv6 interfaces
 * @param runningLocally whether to run in local-mode or not
 * @returns filtered results
 */
export function isUsableResult(result: Interface, runningLocally = false): boolean {
  switch (result.family) {
    case 'IPv6':
      // STUN over IPv6 is not yet supported
      break
    case 'IPv4':
      const u8aAddr = ipToU8aAddress(result.address, 'IPv4')

      if (runningLocally) {
        // Only take local or private addresses
        if (isPrivateAddress(u8aAddr, 'IPv4') || isLocalhost(u8aAddr, 'IPv4')) {
          return true
        }
        break
      }

      // Only take public addresses
      if (!isPrivateAddress(u8aAddr, 'IPv4') && !isLocalhost(u8aAddr, 'IPv4')) {
        return true
      }

      break
  }

  return false
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
  const permutated = function* () {
    if (multiAddrs == undefined || multiAddrs.length == 0) {
      yield* []
      return
    }
    const indices = Array.from({ length: (multiAddrs ?? []).length }, (_, i) => i)

    // Permutates in-place
    randomPermutation(indices)

    for (let i = 0; i < multiAddrs.length; i++) {
      yield multiAddrs[indices[i]]
    }
  }

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

    return await performSTUNRequests(permutated(), socket, undefined, true)
  } else {
    return await performSTUNRequests(
      (function* () {
        if (multiAddrs != undefined && multiAddrs.length > 0) {
          yield* permutated()
        }
        // Fallback option
        yield* PUBLIC_STUN_SERVERS
      })(),
      socket,
      undefined
    )
  }
}
