import { type Socket, createConnection } from 'net'

import { decode, constants, createMessage, createTransaction } from 'stun'

// @ts-ignore untyped module
import retimer from 'retimer'

import { Multiaddr } from '@multiformats/multiaddr'

import { u8aToHex, u8aAddrToString, u8aToNumber, u8aEquals } from '@hoprnet/hopr-utils'
import { STUN_TCP_TIMEOUT } from './constants.js'
import { CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6 } from '../../../constants.js'
import {
  isStunErrorResponse,
  isStunRequest,
  isStunSuccessResponse,
  kStunTypeMask,
  STUN_EXPOSED_CHECK_RESPOSE,
  STUN_QUERY_STATE
} from '../constants.js'

import debug from 'debug'

const log = debug('hopr-connect:stun:tcp')
const error = debug('hopr-connect:stun:tcp:error')
const verbose = debug(`hopr-connect:verbose:stun:tcp`)

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
  state: STUN_QUERY_STATE
}

type Requests = Map<string, Request>

/**
 * Encodes STUN message and sends it to the STUN server using
 * a secondary TCP connection.
 *
 * @param multiaddr address to contact
 * @param tId transactionId, used to distinguish requests
 * @param responsePort port on which the response is expected
 * @param signal used to terminate request
 * @param onUpdate callback, called on incoming responses
 */
function createRequest(
  multiaddr: Multiaddr,
  tId: Buffer,
  responsePort: number | undefined,
  signal: AbortSignal,
  onUpdate: (response: { response?: Interface; transactionId: Buffer }) => void
): void {
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
    // Ask the STUN server to reply on a specific port, see RFC 5780
    message.addAttribute(constants.STUN_ATTR_RESPONSE_PORT, responsePort)
  }

  // Allows multiplexing of STUN protocol with other protocols
  message.addFingerprint()

  let done = false

  // Creates a secondary connection
  const socket = createConnection({
    port: port as number,
    host: address,
    family: 6,
    signal
  })

  const onError = (err: any) => {
    if (done) {
      return
    }
    done = true
    // Ignore timeouts
    if (err.type === 'abort' || err.code === 'ABORT_ERR') {
      verbose(`Timeout while contacting ${address}:${port}`)
    } else {
      error(err)
    }
    onUpdate({ transactionId: tId })
    socket.destroy()
  }

  socket.on('connect', () => {
    socket.write(message.toBuffer(), (err?: any) => {
      if (err) {
        onError(err)
        return
      }

      verbose(
        `STUN request successfully sent to ${address}:${port} Transaction: ${u8aToHex(tId)}${
          responsePort != undefined ? ` port ${responsePort}` : ''
        }`
      )
    })
  })

  socket.on('error', onError)

  socket.on('data', (data: Buffer) => {
    if (done) {
      return
    }
    const response = decode(data)

    switch (response.type & kStunTypeMask) {
      case isStunSuccessResponse:
        onUpdate({
          response: response.getXorAddress() ?? response.getAddress(),
          transactionId: response.transactionId
        })
        if (u8aEquals(response.transactionId, tId)) {
          done = true
          socket.end()
          socket.destroy()
        }
        break
      case isStunRequest:
        // handled by STUN server, ignoring
        break
      case isStunErrorResponse:
        onUpdate({
          transactionId: response.transactionId
        })
        if (u8aEquals(response.transactionId, tId)) {
          done = true
          socket.end()
          socket.destroy()
        }
        break
      default:
        log(`secondary socket: unknown STUN response`, data)
        break
    }
  })
}

/**
 * Creates a STUN request and sends it to the next STUN server and adds
 * the request to the previous requests
 * @param it iterator of STUN servers
 * @param requests map of previous requests
 * @param timeout STUN timeout
 * @param stunPort [optional] port to receive STUN response, supposed to be different from socket port
 * @param onTimeout [optional] called once timeout is due
 * @param onError [optional] called upon errors
 * @param state current state of the request
 * @returns the transaction id
 */
function nextSTUNRequest(
  it: Iterator<Multiaddr>,
  requests: Requests,
  timeout: number,
  stunPort: number | undefined,
  onUpdate: (response: { response?: Interface; transactionId: Buffer }) => void,
  onTimeout: (tId: Buffer) => void,
  onError: (err: any) => void,
  state: STUN_QUERY_STATE
) {
  const chunk = it.next()

  if (chunk.done) {
    onError(Error(`Not enough STUN servers given to determine own public IP address`))
    return
  }

  const nextSTUNRequest = {
    transactionId: createTransaction(),
    multiaddr: chunk.value
  }

  const abort = new AbortController()

  createRequest(chunk.value, nextSTUNRequest.transactionId, stunPort, abort.signal, onUpdate)

  requests.set(u8aToHex(nextSTUNRequest.transactionId), {
    multiaddr: nextSTUNRequest.multiaddr,
    timeout: retimer(() => {
      abort.abort()
      onTimeout(nextSTUNRequest.transactionId)
    }, timeout),
    state
  })

  return nextSTUNRequest.transactionId
}

/**
 * Checks whether host is exposed on TCP listening port, using
 * RFC 5780 STUN protocol.
 * @param multiaddrs list of RFC 5780 STUN servers to contact
 * @param addListener adds a listener to the *existing* TCP socket
 * @param timeout [optional] specify custom timeout
 * @param stunPort port where the response is expected
 * @param runningLocally [optional] local-mode, used for unit and e2e testing
 */
export function isTcpExposedHost(
  multiaddrs: Iterable<Multiaddr>,
  addListener: (listener: (socket: Socket, stream: AsyncIterable<Uint8Array>) => void) => () => void,
  timeout = STUN_TCP_TIMEOUT,
  stunPort: number
): Promise<STUN_EXPOSED_CHECK_RESPOSE> {
  return new Promise<STUN_EXPOSED_CHECK_RESPOSE>(async (resolve) => {
    // Holds sent requests, used to link incoming responses to previous requests
    const requests = new Map<string, Request & { state: STUN_QUERY_STATE }>()

    let removeListener: () => void

    const end = () => {
      removeListener()
    }

    const it = multiaddrs[Symbol.iterator]()

    /**
     * Called once requests on secondary sockets time out.
     * @param transactionId used to link to sent requests
     */
    const onTimeoutSecondary = (transactionId: Buffer) => {
      const tIdString = u8aToHex(transactionId)
      requests.delete(tIdString)

      verbose(`RFC 5780 STUN request on secondary socket timed out, transaction ${tIdString}`)

      nextSTUNRequest(
        it,
        requests,
        timeout,
        undefined,
        updateSecondary,
        onTimeoutSecondary,
        onError,
        STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
      )
    }

    /**
     * Called once request to be received on primary TCP socket times out
     * @param transactionId used to link to sent requests
     */
    const onTimeoutPrimary = (transactionId: Buffer) => {
      const tIdString = u8aToHex(transactionId)
      requests.delete(tIdString)

      log(`RFC 5780 STUN request on primary socket timed out, transaction ${tIdString}`)

      end()
      resolve(STUN_EXPOSED_CHECK_RESPOSE.NOT_EXPOSED)
    }

    /**
     * Called once secondary TCP socket receives a STUN response
     * @param response the STUN response
     */
    const updateSecondary = (response: { response?: Interface; transactionId: Buffer }) => {
      const tIdString = u8aToHex(response.transactionId)

      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(`Received unexpected STUN response. Dropping response, transaction ${tIdString}`)
        return
      }

      request.timeout.clear()
      requests.delete(tIdString)

      if (response.response == undefined) {
        verbose(`Ignoring empty STUN response, transaction ${tIdString}`)
        nextSTUNRequest(
          it,
          requests,
          timeout,
          undefined,
          updateSecondary,
          onTimeoutSecondary,
          onError,
          STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
        )
        return
      }

      switch (request.state) {
        case STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER:
          // STUN server is online, let's see if it supports RESPONSE_PORT extension
          nextSTUNRequest(
            [request.multiaddr][Symbol.iterator](),
            requests,
            timeout,
            stunPort,
            updateSecondary,
            onTimeoutPrimary,
            onError,
            STUN_QUERY_STATE.CHECKING_PORT_MAPPING
          )
          break
        case STUN_QUERY_STATE.CHECKING_PORT_MAPPING:
          // STUN server does not understand RESPONSE_PORT extension
          nextSTUNRequest(
            it,
            requests,
            timeout,
            undefined,
            updateSecondary,
            onTimeoutSecondary,
            onError,
            STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
          )
          break
      }
    }

    /**
     * Called once primary TCP socket receives a STUN response
     * @param response the STUN response
     */
    const updatePrimary = (response: { response?: Interface; transactionId: Buffer }) => {
      const tIdString = u8aToHex(response.transactionId)
      const request = requests.get(tIdString)

      if (request == undefined) {
        verbose(`Received unexpected STUN response. Dropping response`)
        return
      }

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
            undefined,
            updateSecondary,
            onTimeoutSecondary,
            onError,
            STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
          )
          break
      }
    }

    /**
     * Called on errors
     */
    const onError = () => {
      end()
      resolve(STUN_EXPOSED_CHECK_RESPOSE.UNKNOWN)
    }

    /**
     * TCP connection listeners, called once TCP multiplexer detects STUN packets
     * @param _socket not used by this function
     * @param stream packet stream to drain
     */
    const onConnection = async (_socket: Socket, stream: AsyncIterable<Uint8Array>) => {
      for await (const data of stream) {
        const response = decode(
          Buffer.isBuffer(data) ? data : Buffer.from(data.buffer, data.byteOffset, data.byteLength)
        )

        switch (response.type & kStunTypeMask) {
          case isStunSuccessResponse:
            updatePrimary({
              response: response.getXorAddress() ?? response.getAddress(),
              transactionId: response.transactionId
            })
            break
          case isStunRequest:
            // handled by STUN server, ignoring
            break
          default:
            log(`primary socket: unknown STUN response`, data)
            break
        }
      }
    }

    removeListener = addListener(onConnection)

    // Initiate search for usable RFC 5780 STUN server
    nextSTUNRequest(
      it,
      requests,
      timeout,
      undefined,
      updateSecondary,
      onTimeoutSecondary,
      onError,
      STUN_QUERY_STATE.SEARCHING_RFC_5780_STUN_SERVER
    )
  })
}
