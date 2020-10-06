import * as stun from 'webrtc-stun'
import type { Socket, RemoteInfo } from 'dgram'
import Multiaddr from 'multiaddr'
import debug from 'debug'

const verbose = debug('hopr-core:verbose:transport:stun')

export type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

export const STUN_TIMEOUT = 1000

export function handleStunRequest(socket: Socket, data: Buffer, rinfo: RemoteInfo): void {
  verbose('Handle stun request')
  const req = stun.createBlank()

  // if msg is valid STUN message
  if (req.loadBuffer(data)) {
    // if STUN message is BINDING_REQUEST and valid content
    if (req.isBindingRequest({ fingerprint: true })) {
      const res = req
        // prettier-ignore
        .createBindingResponse(true)
        .setXorMappedAddressAttribute(rinfo)
        .setFingerprintAttribute()

      socket.send(res.toBuffer(), rinfo.port, rinfo.address)
    }
  }
}

export function getExternalIp(
  multiAddrs: Multiaddr[],
  socket: Socket
): Promise<{
  port: number
  address: string
}> {
  return new Promise((resolve, reject) => {
    verbose('External IP for', multiAddrs.map((m) => m.toString()).join(','))
    const tids = Array.from({ length: multiAddrs.length }).map(stun.generateTransactionId)

    let result: {
      address: string
      port: number
    }

    let finished = false
    let timeout: NodeJS.Timeout

    const msgHandler = (msg: Buffer) => {
      const res = stun.createBlank()

      if (res.loadBuffer(msg)) {
        let index: number
        if (
          tids.some((tid: string, _index: number) => {
            if (res.isBindingResponseSuccess({ transactionId: tid })) {
              index = _index
              return true
            }

            return false
          })
        ) {
          tids.splice(index, 1)
          const attr = res.getXorMappedAddressAttribute()

          if (attr != null) {
            if (result == null) {
              result = attr
            } else if (tids.length == 0 || attr.port != result.port || attr.address !== result.address) {
              socket.removeListener('message', msgHandler)
              finished = true
              clearTimeout(timeout)
              resolve({
                address: attr.address === result.address ? attr.address : undefined,
                port: attr.port == result.port ? attr.port : undefined,
              })
            }
          }
        }
      }
    }
    socket.on('message', msgHandler)
    socket.on('error', (err) => {
      verbose('Err:', err)
      reject(err)
    })

    multiAddrs.forEach((ma: Multiaddr, index: number) => {
      const nodeAddress = ma.nodeAddress()

      const res = stun
        .createBindingRequest(tids[index])
        //.setSoftwareAttribute(`${pkg.name}@${pkg.version}`)
        .setFingerprintAttribute()

      socket.send(res.toBuffer(), parseInt(nodeAddress.port, 10), nodeAddress.address)
    })

    timeout = setTimeout(() => {
      finished = true
      if (result == null) {
        reject(Error(`Timeout. Could not complete STUN request in time.`))
      } else {
        resolve(result)
      }
    }, STUN_TIMEOUT)
  })
}
