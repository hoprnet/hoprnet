import { createBlank, createBindingRequest, generateTransactionId } from 'webrtc-stun'

import type { Socket, RemoteInfo } from 'dgram'
import { Multiaddr } from 'multiaddr'
import debug from 'debug'
import { randomSubset, ipToU8aAddress, isLocalhost, isPrivateAddress } from '@hoprnet/hopr-utils'
import { CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6 } from '../constants'

const log = debug('hopr-connect:stun:error')
const error = debug('hopr-connect:stun:error')
const verbose = debug('hopr-connect:verbose:stun')

export type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

function isInterface(obj: any): obj is Interface {
  if (obj.family == undefined || obj.port == undefined || obj.address == undefined) {
    return false
  }

  if (!['IPv4', 'IPv6'].includes(obj.family)) {
    return false
  }

  return true
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

/**
 * Handles STUN requests
 * @param socket Node.JS socket to use
 * @param data received packet
 * @param rinfo Addr+Port of the incoming connection
 * @param __fakeRInfo [testing] overwrite incoming information to intentionally send misleading STUN response
 */
export function handleStunRequest(socket: Socket, data: Buffer, rinfo: RemoteInfo, __fakeRInfo?: RemoteInfo): void {
  const req = createBlank()

  // Overwrite console.log because 'webrtc-stun' package
  // pollutes console output
  const consoleBackup = console.log
  console.log = log

  try {
    if (req.loadBuffer(data)) {
      // if STUN message is BINDING_REQUEST and valid content
      if (req.isBindingRequest({ fingerprint: true })) {
        verbose(`Received STUN request from ${rinfo.address}:${rinfo.port}`)

        const res = req
          .createBindingResponse(true)
          .setXorMappedAddressAttribute(__fakeRInfo ?? rinfo)
          .setFingerprintAttribute()

        socket.send(res.toBuffer(), rinfo.port, rinfo.address)
      } else if (!req.isBindingResponseSuccess()) {
        error(`Received a STUN message that is not a binding request. Dropping message.`)
      }
    } else {
      error(`Received a message that is not a STUN message. Dropping message.`)
    }
  } catch (err) {
    consoleBackup(err)
  } finally {
    console.log = consoleBackup
  }
}

export type Request = {
  multiaddr: Multiaddr
  tId: string
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
      tId: generateTransactionId()
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
 * @param usableMultiaddrs multiaddrs to use for STUN requests
 * @param tIds transaction IDs to use, necessary to link requests and responses
 * @param socket the socket to send the STUN requests
 * @returns usable transaction IDs and the corresponding multiaddrs
 */
function sendStunRequests(addrs: Request[], socket: Socket): void {
  for (const addr of addrs) {
    if (![CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6].includes(addr.multiaddr.tuples()[0][0])) {
      error(`Cannot contact STUN server ${addr.multiaddr.toString()} due to invalid address.`)
      continue
    }

    let nodeAddress: ReturnType<Multiaddr['nodeAddress']>
    try {
      nodeAddress = addr.multiaddr.nodeAddress()
    } catch (err) {
      error(err)
      continue
    }

    const res = createBindingRequest(addr.tId).setFingerprintAttribute()

    socket.send(res.toBuffer(), nodeAddress.port, nodeAddress.address, (err?: any) => {
      if (err) {
        error(err.message)
      } else {
        verbose(`STUN request successfully sent to ${nodeAddress.address}:${nodeAddress.port}`)
      }
    })
  }
}

function decodeIncomingSTUNResponses(addrs: Request[], socket: Socket, ms: number = STUN_TIMEOUT): Promise<Request[]> {
  return new Promise<Request[]>((resolve) => {
    let responsesReceived = 0

    let listener: (msg: Buffer) => void
    let finished = false

    let done = () => {
      if (finished) {
        return
      }
      finished = true

      socket.removeListener('message', listener)

      clearTimeout(timeout)

      resolve(addrs)
    }

    const timeout = setTimeout(() => {
      log(
        `STUN timeout. ${addrs.filter((addr) => addr.response).length} of ${
          addrs.length
        } selected STUN servers replied.`
      )
      done()
    }, ms)

    // Receiving a Buffer, not a Uint8Array
    listener = (msg: Buffer) => {
      const res = createBlank()

      // Overwrite console.log because 'webrtc-stun' package
      // pollutes console output
      const consoleBackup = console.log
      console.log = log

      try {
        if (!res.loadBuffer(msg)) {
          error(`Could not decode STUN response`)
          console.log = consoleBackup
          return
        }

        const index: number = addrs.findIndex((addr: Request) =>
          res.isBindingResponseSuccess({ transactionId: addr.tId })
        )

        if (index < 0) {
          error(`Received STUN response with invalid transactionId. Dropping response.`)
          console.log = consoleBackup
          return
        }

        const attr = res.getXorMappedAddressAttribute() ?? res.getMappedAddressAttribute()

        if (!isInterface(attr)) {
          error(`Invalid STUN response. Got ${attr}`)
          return
        }

        console.log = consoleBackup

        if (attr == null) {
          error(`STUN response seems to have neither MappedAddress nor XORMappedAddress set. Dropping message`)
          return
        }

        verbose(`Received STUN response. External address seems to be: ${attr.address}:${attr.port}`)

        if (addrs[index].response != undefined) {
          verbose(`Recieved duplicate response. Dropping message`)
          return
        }

        addrs[index].response = attr
        responsesReceived++

        if (responsesReceived == addrs.length) {
          done()
        }
      } catch (err) {
        consoleBackup(err)
      } finally {
        console.log = consoleBackup
      }
    }

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
 * @param runningLocally set to true when running a local testnet
 */
export async function getExternalIp(
  multiAddrs: Multiaddr[] | undefined,
  socket: Socket,
  runningLocally = false
): Promise<Interface | undefined> {
  let responses: RequestWithResponse[] = []
  if (runningLocally) {
    if (multiAddrs == undefined || multiAddrs.length == 0) {
      const socketAddress = socket.address() as Interface | null
      if (socketAddress == null) {
        throw Error(`Socket is not listening`)
      }
      if (socketAddress.family === 'IPv6') {
        throw Error(`IPv6 is not supported`)
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
