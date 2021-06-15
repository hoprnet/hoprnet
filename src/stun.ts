import * as stun from 'webrtc-stun'

import type { Socket, RemoteInfo } from 'dgram'
import { Multiaddr } from 'multiaddr'
import debug from 'debug'
import { randomSubset } from '@hoprnet/hopr-utils'
import { CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6 } from './constants'

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
 */
export function handleStunRequest(socket: Socket, data: Buffer, rinfo: RemoteInfo): void {
  const req = stun.createBlank()

  // Overwrite console.log because 'webrtc-stun' package
  // pollutes console output
  const backup = console.log
  console.log = log

  if (req.loadBuffer(data)) {
    // if STUN message is BINDING_REQUEST and valid content
    if (req.isBindingRequest({ fingerprint: true })) {
      verbose(`Received STUN request from ${rinfo.address}:${rinfo.port}`)

      const res = req.createBindingResponse(true).setXorMappedAddressAttribute(rinfo).setFingerprintAttribute()

      socket.send(res.toBuffer(), rinfo.port, rinfo.address)
    } else if (!req.isBindingResponseSuccess()) {
      error(`Received a STUN message that is not a binding request. Dropping message.`)
    }
  } else {
    error(`Received a message that is not a STUN message. Dropping message.`)
  }
  console.log = backup
}

/**
 * Tries to determine the external IPv4 address
 * @returns Addrs+Port or undefined if the STUN response are ambigous (e.g. bidirectional NAT)
 *
 * @param multiAddrs Multiaddrs to use as STUN servers
 * @param socket Node.JS socket to use for the STUN request
 */
export function getExternalIp(
  multiAddrs: Multiaddr[] | undefined,
  socket: Socket
): Promise<ConnectionInfo | undefined> {
  return new Promise<ConnectionInfo | undefined>(async (resolve, reject) => {
    let usableMultiaddrs: Multiaddr[]
    if (multiAddrs == undefined || multiAddrs.length == 0) {
      usableMultiaddrs = randomSubset(PUBLIC_STUN_SERVERS, DEFAULT_PARALLEL_STUN_CALLS)
    } else {
      usableMultiaddrs = multiAddrs
    }

    verbose(`Getting external IP by using ${usableMultiaddrs.map((m) => m.toString()).join(',')}`)
    const tids = Array.from({ length: usableMultiaddrs.length }).map(stun.generateTransactionId)

    const results: ConnectionInfo[] = []

    let timeout: NodeJS.Timeout

    const done = () => {
      socket.removeListener('message', msgHandler)

      clearTimeout(timeout)

      if (results.length == 0) {
        error(`STUN Timeout. Could not complete STUN request in time.`)
        resolve(undefined)
        return
      }

      if (results.length == 1) {
        resolve(results[0])
        return
      }

      for (const result of results) {
        if (result.port != results[0].port) {
          return resolve(undefined)
        }
      }

      resolve(results[0])
    }

    const msgHandler = (msg: Buffer) => {
      const res = stun.createBlank()

      // Overwrite console.log because 'webrtc-stun' package
      // pollutes console output
      const backup = console.log
      console.log = log

      if (!res.loadBuffer(msg)) {
        error(`Could not decode STUN response`)
        console.log = backup
        return
      }

      const index: number = tids.findIndex((tid: string) => res.isBindingResponseSuccess({ transactionId: tid }))

      if (index < 0) {
        error(`Received STUN response with invalid transactionId. Dropping response.`)
        console.log = backup
        return
      }

      tids.splice(index, 1)
      const attr = res.getXorMappedAddressAttribute() ?? res.getMappedAddressAttribute()

      console.log = backup

      if (attr == null) {
        error(`STUN response seems to have neither MappedAddress nor XORMappedAddress set. Dropping message`)
        return
      }

      verbose(`Received STUN response. External address seems to be: ${attr.address}:${attr.port}`)

      if (results.push(attr) == usableMultiaddrs.length) {
        done()
      }
    }

    socket.on('message', msgHandler)

    const allSent = performStunRequests(usableMultiaddrs, tids, socket)

    if (allSent.length > 0) {
      const sendResults = await Promise.all(allSent)

      if (!sendResults.some((result) => result)) {
        reject(
          new Error(
            `Cannot send any STUN packets. Tried with: ${usableMultiaddrs
              .map((ma: Multiaddr) => ma.toString())
              .join(', ')}`
          )
        )
      }

      timeout = setTimeout(() => done(), STUN_TIMEOUT)
    } else {
      reject(new Error(`Cannot detect external IP address using STUN`))
    }
  })
}

function performStunRequests(usableMultiaddrs: Multiaddr[], tIds: string[], socket: Socket): Promise<boolean>[] {
  const result: Promise<boolean>[] = []

  for (const [index, ma] of usableMultiaddrs.entries()) {
    if (![CODE_IP4, CODE_IP6, CODE_DNS4, CODE_DNS6].includes(ma.tuples()[0][0])) {
      error(`Cannot contact STUN server ${ma.toString()} due invalid address.`)
      continue
    }

    const nodeAddress = ma.nodeAddress()

    const res = stun.createBindingRequest(tIds[index]).setFingerprintAttribute()

    verbose(`STUN request sent to ${nodeAddress.address}:${nodeAddress.port}`)

    result.push(
      new Promise<boolean>((resolve) => {
        socket.send(res.toBuffer(), nodeAddress.port, nodeAddress.address, (err: any) => {
          if (err) {
            resolve(false)
          } else {
            resolve(true)
          }
        })
      })
    )
  }

  return result
}
