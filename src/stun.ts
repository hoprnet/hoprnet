import * as stun from 'webrtc-stun'

import type { Socket, RemoteInfo } from 'dgram'
import Multiaddr from 'multiaddr'
import debug from 'debug'

const error = debug('hopr-connect:error')
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
  Multiaddr(`/dns4/stun.sipgate.net/udp/3478`),
  Multiaddr(`/dns4/stun.callwithus.com/udp/3478`),
  Multiaddr(`/dns4/stun.counterpath.net/udp/3478`)
]

export function handleStunRequest(socket: Socket, data: Buffer, rinfo: RemoteInfo): void {
  const req = stun.createBlank()

  // Overwrite console.log because 'webrtc-stun' package
  // pollutes console output
  const backup = console.log
  console.log = () => {}

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

export function getExternalIp(multiAddrs: Multiaddr[] | undefined, socket: Socket): Promise<ConnectionInfo> {
  return new Promise<ConnectionInfo>((resolve, reject) => {
    if (multiAddrs == undefined || multiAddrs.length == 0) {
      multiAddrs = PUBLIC_STUN_SERVERS
    }

    verbose(`Getting external IP by using ${multiAddrs.map((m) => m.toString()).join(',')}`)
    const tids = Array.from({ length: multiAddrs.length }).map(stun.generateTransactionId)

    let result: ConnectionInfo

    // @TODO add assert call
    // let _finished = false
    let timeout: NodeJS.Timeout

    const msgHandler = (msg: Buffer) => {
      const res = stun.createBlank()

      // Overwrite console.log because 'webrtc-stun' package
      // pollutes console output
      const backup = console.log
      console.log = () => {}

      if (res.loadBuffer(msg)) {
        let index: number = tids.findIndex((tid: string) => {
          if (res.isBindingResponseSuccess({ transactionId: tid })) {
            return true
          }

          return false
        })

        if (index >= 0) {
          tids.splice(index, 1)
          const attr = res.getXorMappedAddressAttribute() || res.getMappedAddressAttribute()

          if (attr != null) {
            verbose(`Received STUN response. External address seems to be: ${attr.address}:${attr.port}`)

            if (result == undefined) {
              result = attr
            }

            if (tids.length == 0 || attr.port != result.port || attr.address !== result.address) {
              socket.removeListener('message', msgHandler)
              // @TODO add assert call
              // _finished = true
              clearTimeout(timeout)

              if (attr.address !== result.address || attr.port != result.port) {
                reject()
              }

              resolve({
                address: attr.address,
                port: attr.port
              })
            }
          } else {
            error(`STUN response seems to have neither MappedAddress nor XORMappedAddress set. Dropping message`)
          }
        } else {
          error(`Received STUN response with invalid transactionId. Dropping response.`)
        }
      }
      console.log = backup
    }
    socket.on('message', msgHandler)
    socket.on('error', (err) => {
      verbose('Err:', err)
      reject(err)
    })

    multiAddrs.forEach((ma: Multiaddr, index: number) => {
      if (!['ip4', 'ip6', 'dns4', 'dns6'].includes(ma.protoNames()[0])) {
        error(`Cannot contact STUN server ${ma.toString()} because the host is unknown.`)
        return
      }

      const nodeAddress = ma.nodeAddress()

      const res = stun.createBindingRequest(tids[index]).setFingerprintAttribute()

      verbose(`STUN request sent to ${nodeAddress.address}:${nodeAddress.port}`)

      socket.send(res.toBuffer(), nodeAddress.port as any, nodeAddress.address)
    })

    timeout = setTimeout(() => {
      socket.removeListener('message', msgHandler)
      // @TODO add assert call
      // _finished = true
      if (result == undefined) {
        reject(Error(`Timeout. Could not complete STUN request in time.`))
      } else {
        resolve(result)
      }
    }, STUN_TIMEOUT)
  })
}
