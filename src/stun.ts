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

type ConnectionInfo = {
  port: number
  address: string
}

export const STUN_TIMEOUT = 1000

export const PUBLIC_STUN_SERVERS = [
  Multiaddr(`/dns4/stun.sipgate.net/udp/3478`),
  Multiaddr(`/dns4/stun.callwithus.com/udp/3478`),
  Multiaddr(`/dns4/stun.counterpath.net/udp/3478`)
]

export function handleStunRequest(socket: Socket, data: Buffer, rinfo: RemoteInfo): void {
  const req = stun.createBlank()

  const backup = console.log
  console.log = () => {}
  // if msg is valid STUN message
  if (req.loadBuffer(data)) {
    // if STUN message is BINDING_REQUEST and valid content
    if (req.isBindingRequest({ fingerprint: true })) {
      verbose(`stun request received`, rinfo.address, rinfo.port)

      const res = req.createBindingResponse(true).setXorMappedAddressAttribute(rinfo).setFingerprintAttribute()

      socket.send(res.toBuffer(), rinfo.port, rinfo.address)
    }
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

      const backup = console.log
      console.log = () => {}

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
          verbose(`stun response received`)
          tids.splice(index!, 1)
          const attr = res.getXorMappedAddressAttribute() || res.getMappedAddressAttribute()

          if (attr != undefined) {
            if (result == undefined) {
              result = attr
            } else if (tids.length == 0 || attr.port != result.port || attr.address !== result.address) {
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
          }
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
        return
      }

      const nodeAddress = ma.nodeAddress()

      const res = stun
        .createBindingRequest(tids[index])
        //.setSoftwareAttribute(`${pkg.name}@${pkg.version}`)
        .setFingerprintAttribute()

      verbose(`STUN request sent`, nodeAddress)
      socket.send(res.toBuffer(), parseInt(nodeAddress.port, 10), nodeAddress.address)
    })

    timeout = setTimeout(() => {
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
