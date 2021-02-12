import * as stun from 'webrtc-stun'

import type { Socket, RemoteInfo } from 'dgram'
import Multiaddr from 'multiaddr'
import debug from 'debug'
import { randomSubset } from '@hoprnet/hopr-utils'

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
  Multiaddr(`/dns4/stun.l.google.com/udp/19302`),
  Multiaddr(`/dns4/stun1.l.google.com/udp/19302`),
  Multiaddr(`/dns4/stun2.l.google.com/udp/19302`),
  Multiaddr(`/dns4/stun3.l.google.com/udp/19302`),
  Multiaddr(`/dns4/stun4.l.google.com/udp/19302`),
  Multiaddr(`/dns4/stun.sipgate.net/udp/3478`),
  Multiaddr(`/dns4/stun.callwithus.com/udp/3478`),
  Multiaddr(`/dns4/stun.counterpath.net/udp/3478`)
]

export const DEFAULT_PARALLEL_STUN_CALLS = 4

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

export function getExternalIp(
  multiAddrs: Multiaddr[] | undefined,
  socket: Socket
): Promise<ConnectionInfo | undefined> {
  return new Promise<ConnectionInfo | undefined>((resolve) => {
    if (multiAddrs == undefined || multiAddrs.length == 0) {
      multiAddrs = randomSubset(PUBLIC_STUN_SERVERS, DEFAULT_PARALLEL_STUN_CALLS)
    }

    verbose(`Getting external IP by using ${multiAddrs.map((m) => m.toString()).join(',')}`)
    const tids = Array.from({ length: multiAddrs.length }).map(stun.generateTransactionId)

    const results: ConnectionInfo[] = []

    // @TODO add assert call
    // let _finished = false
    let timeout: NodeJS.Timeout

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

      if (attr == null) {
        error(`STUN response seems to have neither MappedAddress nor XORMappedAddress set. Dropping message`)
        console.log = backup
        return
      }

      verbose(`Received STUN response. External address seems to be: ${attr.address}:${attr.port}`)

      if (results.push(attr) == multiAddrs?.length) {
        done()
      }

      console.log = backup
    }

    const errHandler = (err: any) => {
      verbose('STUN error', err)
    }

    socket.on('message', msgHandler)
    socket.on('error', errHandler)

    multiAddrs.forEach((ma: Multiaddr, index: number) => {
      if (!['ip4', 'ip6', 'dns4', 'dns6'].includes(ma.protoNames()[0])) {
        error(`Cannot contact STUN server ${ma.toString()} due invalid address.`)
        return
      }

      const nodeAddress = ma.nodeAddress()

      const res = stun.createBindingRequest(tids[index]).setFingerprintAttribute()

      verbose(`STUN request sent to ${nodeAddress.address}:${nodeAddress.port}`)

      socket.send(res.toBuffer(), nodeAddress.port as any, nodeAddress.address)
    })

    const done = () => {
      socket.removeListener('message', msgHandler)
      socket.removeListener('error', errHandler)

      clearTimeout(timeout)

      if (results.length == 0) {
        error(`STUN Timeout. Could not complete STUN request in time.`)
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

    timeout = setTimeout(() => done(), STUN_TIMEOUT)
  })
}
