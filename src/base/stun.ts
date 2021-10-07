import * as stun from 'webrtc-stun'

import type { Socket, RemoteInfo } from 'dgram'
import { Multiaddr } from 'multiaddr'
import debug from 'debug'
import { randomSubset } from '@hoprnet/hopr-utils'
import { CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6 } from '../constants'
import { ipToU8aAddress, isLocalhost, isPrivateAddress } from '../utils'

const log = debug('hopr-connect:stun:error')
const error = debug('hopr-connect:stun:error')
const verbose = debug('hopr-connect:verbose:stun')

export type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

type ConnectionInfo = {
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

/**
 * Handle STUN requests
 * @param socket Node.JS socket to use
 * @param data received packet
 * @param rinfo Addr+Port of the incoming connection
 * @param __fakeRInfo [testing] overwrite incoming information to intentionally send misleading STUN response
 */
export function handleStunRequest(socket: Socket, data: Buffer, rinfo: RemoteInfo, __fakeRInfo?: RemoteInfo): void {
  const req = stun.createBlank()

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

type Address = {
  address: string
  family: string
  port: number
}

type Request = {
  multiaddr: Multiaddr
  tId: string
  failed?: boolean
  response?: Address
}
/**
 * Tries to determine the external IPv4 address
 * @returns Addrs+Port or undefined if the STUN response are ambiguous (e.g. bidirectional NAT)
 *
 * @param multiAddrs Multiaddrs to use as STUN servers
 * @param socket Node.JS socket to use for the STUN request
 * @param runningLocally set to true when running a local testnet
 */
export async function getExternalIp(
  multiAddrs: Multiaddr[] | undefined,
  socket: Socket,
  runningLocally = false
): Promise<ConnectionInfo | undefined> {
  let usableMultiaddrs: Multiaddr[]
  let usingPublicServers = false

  if (multiAddrs == undefined || multiAddrs.length == 0) {
    if (runningLocally) {
      // Do not try to contact public STUN servers when running locally
      return
    }
    // Use public STUN servers if no own STUN servers are given
    usingPublicServers = true
    usableMultiaddrs = randomSubset(PUBLIC_STUN_SERVERS, DEFAULT_PARALLEL_STUN_CALLS)
    verbose(`No own STUN servers given. Using ${usableMultiaddrs.map((ma: Multiaddr) => ma.toString()).join(', ')}`)
  } else if (multiAddrs.length > DEFAULT_PARALLEL_STUN_CALLS) {
    // Limit number of STUN servers to contact
    usableMultiaddrs = randomSubset(multiAddrs, DEFAULT_PARALLEL_STUN_CALLS)
  } else {
    usableMultiaddrs = multiAddrs
  }

  verbose(`Trying to determine external IP by using ${usableMultiaddrs.map((m) => m.toString()).join(',')}`)

  let responses = await performSTUNRequests(usableMultiaddrs, socket, STUN_TIMEOUT)

  if (responses.length == 0 && (usingPublicServers || runningLocally)) {
    // We have already contacted public and this did not lead to a result
    // hence we cannot determine public IP address
    return
  }

  if ([0, 1].includes(responses.length) && !runningLocally && !usingPublicServers) {
    // Received too less results to see if addresses are ambiguous,
    // let's try some public servers
    usingPublicServers = true
    usableMultiaddrs = randomSubset(PUBLIC_STUN_SERVERS, DEFAULT_PARALLEL_STUN_CALLS)
    verbose(
      `Using own STUN servers did not lead to a result. Trying ${usableMultiaddrs
        .map((ma: Multiaddr) => ma.toString())
        .join(', ')}`
    )

    responses.push(...(await performSTUNRequests(usableMultiaddrs, socket, STUN_TIMEOUT)))
  }

  if (responses.length == 0) {
    // Even contacting public STUN servers did not lead
    return
  }

  let filteredResults = getUsableResults(responses, runningLocally)

  if ([0, 1].includes(filteredResults.length) && !runningLocally && !usingPublicServers) {
    // Received results were not usable to determine public IP address
    // now trying public ones
    verbose(
      `Using own STUN servers did not lead to a result. Trying ${usableMultiaddrs
        .map((ma: Multiaddr) => ma.toString())
        .join(', ')}`
    )
    usingPublicServers = true
    usableMultiaddrs = randomSubset(PUBLIC_STUN_SERVERS, DEFAULT_PARALLEL_STUN_CALLS)
    responses.push(...(await performSTUNRequests(usableMultiaddrs, socket, STUN_TIMEOUT)))

    filteredResults = getUsableResults(responses, runningLocally)
  }

  const interpreted = intepreteResults(filteredResults)

  if (interpreted.ambiguous) {
    return undefined
  } else {
    return interpreted.publicAddress
  }
}

async function performSTUNRequests(
  multiAddrs: Multiaddr[],
  socket: Socket,
  timeout = STUN_TIMEOUT
): Promise<Request[]> {
  let requests = generateRequests(multiAddrs)
  let results = decodeIncomingSTUNResponses(requests, socket, timeout)
  sendStunRequests(requests, socket)

  return (await results).filter((request: Request) => request.response)
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
      error(`Cannot contact STUN server ${addr.multiaddr.toString()} due invalid address.`)
      continue
    }

    let nodeAddress: ReturnType<Multiaddr['nodeAddress']>
    try {
      nodeAddress = addr.multiaddr.nodeAddress()
    } catch (err) {
      error(err)
      continue
    }

    const res = stun.createBindingRequest(addr.tId).setFingerprintAttribute()

    verbose(`STUN request sent to ${nodeAddress.address}:${nodeAddress.port}`)
    socket.send(res.toBuffer(), nodeAddress.port, nodeAddress.address, (err: any) => err && error(err.message))
  }
}

function decodeIncomingSTUNResponses(addrs: Request[], socket: Socket, ms: number = STUN_TIMEOUT) {
  return new Promise<Request[]>((resolve) => {
    let responsesReceived = 0

    let done: () => void
    let listener: (msg: Buffer) => void
    let finished = false

    const timeout = setTimeout(() => {
      log(`STUN timeout. None of the selected STUN servers replied.`)
      done()
    }, ms)

    done = () => {
      if (finished) {
        return
      }
      finished = true

      socket.removeListener('message', listener)

      clearTimeout(timeout)

      resolve(addrs)
    }

    listener = (msg: Buffer) => {
      const res = stun.createBlank()

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

function getUsableResults(results: Request[], runningLocally = false): Request[] {
  let filtered: Request[] = []

  for (const result of results) {
    if (!result.response) {
      continue
    }

    switch (result.response.family) {
      case 'IPv6':
        // STUN over IPv6 is not yet supported
        break
      case 'IPv4':
        const u8aAddr = ipToU8aAddress(result.response.address, 'IPv4')
        // Disitinguish two use cases:
        // Unit tests:
        // - run several instances on one machine, hence STUN response is expected to be
        //   'localhost:somePort'
        // CI tests / large E2E tests:
        // - run several instances on multiple machines running in the *same* local network
        //   hence STUN response is expected to be a local address
        // Disclaimer:
        // the mixed use case, meaning some instances running on the same machine and some
        // instances running on machines in the same network is not expected
        if ((isPrivateAddress(u8aAddr, 'IPv4') || isLocalhost(u8aAddr, 'IPv4')) == runningLocally) {
          filtered.push(result)
        }
        break
      default:
        error(`Invalid STUN response. Got family: ${result.response.family}`)
        break
    }
  }

  return filtered
}

function intepreteResults(results: Request[]):
  | {
      ambiguous: true
    }
  | {
      ambiguous: false
      publicAddress: Address
    } {
  if (results.length == 0 || results[0].response == undefined) {
    return { ambiguous: true }
  }
  const ambiguous = results
    .slice(1)
    .some(
      (req: Request) =>
        req.response?.address !== results[0].response?.address || req.response?.port != results[0].response?.port
    )

  if (ambiguous) {
    return { ambiguous }
  } else {
    return { ambiguous, publicAddress: results[0].response }
  }
}

function generateRequests(multiAddrs: Multiaddr[]): Request[] {
  return multiAddrs.map<Request>((multiaddr: Multiaddr) => ({
    multiaddr,
    tId: stun.generateTransactionId()
  }))
}
