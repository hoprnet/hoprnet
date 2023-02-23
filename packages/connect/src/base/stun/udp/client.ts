import { type Socket, createSocket, type RemoteInfo } from 'dgram'
import debug from 'debug'
import { Multiaddr } from '@multiformats/multiaddr'
import { decode, constants, createMessage, createTransaction } from 'stun'

import { isStun } from '../../../utils/index.js'

// @ts-ignore untyped module
import retimer from 'retimer'

import { u8aToHex, ipToU8aAddress, isAvadoPrivateNetwork, isPrivateAddress, isLocalhost } from '@hoprnet/hopr-utils'

import {
  isStunErrorResponse,
  isStunRequest,
  isStunSuccessResponse,
  kStunTypeMask,
  STUN_EXPOSED_CHECK_RESPOSE,
  STUN_QUERY_STATE
} from '../constants.js'
import { STUN_UDP_TIMEOUT } from './constants.js'
import { ip6Lookup } from '../../../utils/index.js'
import { parseStunAddress } from '../utils.js'

const log = debug('hopr-connect:stun:udp')
const error = debug('hopr-connect:stun:udp:error')
const verbose = debug('hopr-connect:verbose:udp:stun')

type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

type Request = {
  multiaddr: Multiaddr
  responsePort?: number
  response?: Interface
  timeout: any
}

type Requests = Map<string, Request>

/**
 * Checks if two given endpoints are the same
 * @param first first interface
 * @param second second interface
 * @returns true if interface are equal
 */
function sameEndpoint(first: Interface, second: Interface): boolean {
  return first.family === second.family && first.address === second.address && first.port == second.port
}

/**
 * Checks if a decoded STUN response matches a previous STUN request
 * @param requests map of previous requests
 * @param response incoming, decoded STUN response
 * @returns true if incoming response matches an existing request
 */
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
 * Creates a STUN request and sends it to the next STUN server and adds
 * the request to the previous requests
 * @param it iterator of STUN servers
 * @param requests map of previous requests
 * @param timeout STUN timeout
 * @param socket port to use for sending STUN request
 * @param stunPort [optional] port to receive STUN response, supposed to be different from socket port
 * @param onTimeout [optional] called once timeout is due
 * @param onError [optional] called upon errors
 * @param state current state of the request
 * @returns the transaction id
 */
function nextSTUNRequest(
  it: Iterator<Multiaddr>,
  requests: Map<string, Request & { state?: STUN_QUERY_STATE }>,
  timeout: number,
  socket: Socket,
  stunPort: number | undefined,
  onTimeout: (tId: Buffer) => void,
  onError: (err: any) => void,
  state?: STUN_QUERY_STATE
): [Buffer, Multiaddr] | undefined {
  const chunk = it.next()

  if (chunk.done) {
    onError(Error(`Not enough STUN servers given to determine own public IP address`))
    return
  }

  const nextSTUNRequest = {
    transactionId: createTransaction(),
    multiaddr: chunk.value
  }
  requests.set(u8aToHex(nextSTUNRequest.transactionId), {
    multiaddr: nextSTUNRequest.multiaddr,
    timeout: retimer(onTimeout, timeout, nextSTUNRequest.transactionId),
    state
  })
  sendStunRequest(nextSTUNRequest.multiaddr, nextSTUNRequest.transactionId, stunPort, socket)

  return [nextSTUNRequest.transactionId, nextSTUNRequest.multiaddr]
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
  update: (response: { response?: Interface; transactionId: Buffer }) => void
): () => void {
  const listener = (data: Buffer | Uint8Array, rinfo: RemoteInfo) => {
    if (!isStun(data)) {
      return
    }

    const response = decode(Buffer.isBuffer(data) ? data : Buffer.from(data.buffer, data.byteOffset, data.byteLength))

    switch (response.type & kStunTypeMask) {
      case isStunSuccessResponse:
        log(`received STUN response from ${rinfo.address}:${rinfo.port}`)
        update({
          response: response.getXorAddress() ?? response.getAddress(),
          transactionId: response.transactionId
        })
        break
      case isStunRequest:
        log(`client: received STUN request`)
        // handled by STUN server, ignoring
        break
      case isStunErrorResponse:
        update({
          transactionId: response.transactionId
        })
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
 * Encodes STUN message and sends it using the given socket
 * to a STUN server.
 * @param multiaddr address to contact
 * @param tId
 * @param socket socket to send the STUN requests
 */
function sendStunRequest(multiaddr: Multiaddr, tId: Buffer, responsePort: number | undefined, socket: Socket): void {
  const { address, port } = parseStunAddress(multiaddr)

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
        `STUN request successfully sent to ${address}:${port}, transaction ${u8aToHex(tId)}${
          responsePort != undefined ? ` port ${responsePort}` : ''
        }`
      )
    }
  })
}

/**
 * Filters STUN responses according to network situation, e.g. local testnet
 * @dev Drops IPv6 responses because IPv6 is not yet supported
 * IPv6 interfaces
 * @param runningLocally whether to run in local-mode or not
 * @returns filtered results
 */
function isUsableResult(result: Interface, runningLocally = false): boolean {
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

      if ((process.env.AVADO ?? 'false').toLowerCase() === 'true' && isAvadoPrivateNetwork(u8aAddr, 'IPv4')) {
        // Even if we find a STUN server within our DAppnode or AVADO network,
        // the perceived internal container address is not public despite it looks like that
        return false
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
  timeout = STUN_UDP_TIMEOUT,
  runningLocally = false
): Promise<[Interface | undefined, Multiaddr[]]> {
  return new Promise<[Interface | undefined, Multiaddr[]]>((resolve, reject) => {
    let successfulResponses = 0
    const requests: Requests = new Map<string, Request>()

    const it = multiaddrs[Symbol.iterator]()

    let stopListening: () => void

    const usedStunServers: Multiaddr[] = []

    /**
     * Called once a STUN request times out.
     * Used to issue a new request.
     */
    const onTimeout = (transactionId: Buffer) => {
      requests.delete(u8aToHex(transactionId))
      const result = nextSTUNRequest(it, requests, timeout, socket, undefined, onTimeout, onError)
      if (result != undefined) {
        usedStunServers.push(result[1])
      }
    }

    /**
     * Called on received STUN responses.
     * Validates response and issues a new request if necessary
     * @param response the STUN response
     */
    const update = (response: { response?: Interface; transactionId: Buffer }) => {
      const tIdString = u8aToHex(response.transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(`Received unexpected STUN response. Dropping response`)
        return
      }

      request.timeout.clear()
      if (response.response == undefined || !isUsableResult(response.response, runningLocally)) {
        requests.delete(tIdString)

        const result = nextSTUNRequest(it, requests, timeout, socket, undefined, onTimeout, onError)
        if (result != undefined) {
          usedStunServers.push(result[1])
        }
        return
      }

      request.response = response.response
      requests.set(tIdString, request)

      successfulResponses++
      if (successfulResponses == 2) {
        stopListening()
        resolve([
          sameResponse(requests, response as Required<typeof response>) != undefined ? response.response : undefined,
          usedStunServers
        ])
        return
      }
    }

    stopListening = decodeIncomingSTUNResponses(socket, update)

    const onError = (err: any) => {
      stopListening()
      reject(err)
    }

    let result: [Buffer, Multiaddr] | undefined
    for (let i = 0; i < 2; i++) {
      result = nextSTUNRequest(it, requests, timeout, socket, undefined, onTimeout, onError)
      if (result != undefined) {
        // Reuse servers for RFC 5780 STUN requests later
        usedStunServers.push(result[1])
      }
    }
  })
}

/**
 * Checks whether the nodes' interface is reachable from the internet
 * @param multiaddrs usable STUN servers
 * @param socket UDP socket to send requests
 * @param timeout
 * @param stunPort [optional] port to receive the STUN response
 * @param runningLocally [optional] use for e2e tests
 * @returns
 */
export function isUdpExposedHost(
  multiaddrs: Iterable<Multiaddr>,
  socket: Socket,
  timeout = STUN_UDP_TIMEOUT,
  stunPort = socket.address().port,
  runningLocally = false
): Promise<STUN_EXPOSED_CHECK_RESPOSE> {
  return new Promise<STUN_EXPOSED_CHECK_RESPOSE>(async (resolve) => {
    const requests = new Map<string, Request & { state: STUN_QUERY_STATE }>()

    const secondarySocket = createSocket({
      type: 'udp6',
      lookup: ip6Lookup
    })

    let stopListening: () => void
    let stopListeningSecondary: () => void

    const end = () => {
      stopListening()
      stopListeningSecondary()
      secondarySocket.close()
    }

    const onError = (err: any) => {
      end()
      error(err)
      resolve(STUN_EXPOSED_CHECK_RESPOSE.UNKNOWN)
    }

    secondarySocket.on('error', onError)

    await new Promise<void>((resolve) => secondarySocket.bind(resolve))

    const [secondaryInterface, usedStunServers] = await performSTUNRequests(
      multiaddrs,
      secondarySocket,
      timeout,
      runningLocally
    )

    if (secondaryInterface == undefined) {
      // Endpoint-dependent mapping, most likely bidirectional NAT
      resolve(STUN_EXPOSED_CHECK_RESPOSE.NOT_EXPOSED)
      return
    }

    const it = (function* () {
      yield* usedStunServers
      yield* multiaddrs
    })()

    /**
     * Called onces a request sent *from* the secondary socket times out.
     * Will issue a new STUN request
     * @param transactionId identifier of the STUN request
     */
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
        STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
      )
    }

    /**
     * Called when giving up waiting for a STUN response on primary socket
     * @param transactionId identifier of the STUN request
     */
    const onTimeoutPrimary = (transactionId: Buffer) => {
      const tIdString = u8aToHex(transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(`Received unexpected STUN response. Dropping response`)
        return
      }

      end()
      resolve(STUN_EXPOSED_CHECK_RESPOSE.NOT_EXPOSED)
      return
    }

    /**
     * Called when receiving a response on the secondary socket
     * Issues a new request if unexpected response or response on the wrong socket.
     * Escalates request if RFC 5780 seems to be supported by STUN server
     * @param response the STUN response, including transactionId
     */
    const updateSecondary = (response: { response?: Interface; transactionId: Buffer }) => {
      const tIdString = u8aToHex(response.transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(`Received unexpected STUN response. Dropping response`)
        return
      }

      log(`Received RFC 5780 STUN response on secondary interface, transaction ${tIdString}`)

      request.timeout.clear()
      requests.delete(tIdString)

      if (response.response == undefined) {
        nextSTUNRequest(
          it,
          requests,
          timeout,
          secondarySocket,
          secondaryInterface.port,
          onTimeoutSecondary,
          onError,
          STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
        )
        return
      }

      switch (request.state) {
        case STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER:
          nextSTUNRequest(
            [request.multiaddr][Symbol.iterator](),
            requests,
            timeout,
            secondarySocket,
            stunPort,
            onTimeoutPrimary,
            onError,
            STUN_QUERY_STATE.CHECKING_PORT_MAPPING
          )
          break
        case STUN_QUERY_STATE.CHECKING_PORT_MAPPING:
          verbose(
            `Received unexpected RFC 5780 response on secondary port, server does not support RESPONSE_PORT extension, transaction ${tIdString}`
          )
          nextSTUNRequest(
            it,
            requests,
            timeout,
            secondarySocket,
            secondaryInterface.port,
            onTimeoutSecondary,
            onError,
            STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
          )
          break
      }
    }

    /**
     * Called when receiving a STUN response on primary socket
     * @param response the STUN response
     */
    const updatePrimary = (response: { response?: Interface; transactionId: Buffer }) => {
      const tIdString = u8aToHex(response.transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(`Received unexpected STUN response. Dropping response`)
        return
      }

      verbose(`Received RFC 5780 STUN response, transaction ${tIdString}`)

      request.timeout.clear()
      requests.delete(tIdString)

      switch (request.state) {
        case STUN_QUERY_STATE.CHECKING_PORT_MAPPING:
          end()
          resolve(STUN_EXPOSED_CHECK_RESPOSE.EXPOSED)
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
            onError,
            STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
          )
          break
      }
    }

    stopListening = decodeIncomingSTUNResponses(socket, updatePrimary)
    stopListeningSecondary = decodeIncomingSTUNResponses(secondarySocket, updateSecondary)

    // Initiate requests
    nextSTUNRequest(
      it,
      requests,
      timeout,
      secondarySocket,
      secondaryInterface.port,
      onTimeoutSecondary,
      onError,
      STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
    )
  })
}
