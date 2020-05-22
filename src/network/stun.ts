import dgram, { Socket } from 'dgram'
import * as stun from 'webrtc-stun'
import { HoprOptions } from '..'

import { DEFAULT_STUN_PORT } from '../constants'

export type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

class Stun {
  private socket: Socket

  constructor(private options: HoprOptions) {}

  static getExternalIP(address: { hostname: string; port: number }, usePort?: number): Promise<Interface> {
    return new Promise<Interface>(async (resolve, reject) => {
      const socket = dgram.createSocket({ type: 'udp4' })
      const tid = stun.generateTransactionId()

      if (usePort !== undefined) {
        await bindSocketToPort(socket, usePort)
      }

      socket.on('message', async msg => {
        const res = stun.createBlank()

        if (res.loadBuffer(msg)) {
          if (res.isBindingResponseSuccess({ transactionId: tid })) {
            const attr = res.getXorMappedAddressAttribute() as Interface

            if (attr) {
              await releaseSocketFromPort(socket)

              resolve(attr)
            }
          }
        }

      })

      const req = stun.createBindingRequest(tid).setFingerprintAttribute()

      socket.send(req.toBuffer(), address.port, address.hostname)
    })
  }

  getSocket() {
    if (this.options.hosts.ip4 !== undefined && this.options.hosts.ip6 !== undefined) {
      return dgram.createSocket({ type: 'udp6' })
    } else if (this.options.hosts.ip4 !== undefined) {
      return dgram.createSocket({ type: 'udp4' })
    } else if (this.options.hosts.ip6 !== undefined) {
      return dgram.createSocket({ type: 'udp6', ipv6Only: true })
    }
  }

  async startServer() {
    return new Promise(async (resolve, reject) => {
      this.socket = this.getSocket()

      this.socket.on('message', (msg, rinfo) => {
        const req = stun.createBlank()

        // if msg is valid STUN message
        if (req.loadBuffer(msg)) {
          // if STUN message is BINDING_REQUEST and valid content
          if (req.isBindingRequest({ fingerprint: true })) {
            const res = req.createBindingResponse(true).setXorMappedAddressAttribute(rinfo).setFingerprintAttribute()

            this.socket.send(res.toBuffer(), rinfo.port, rinfo.address)
          }
        }
      })

      resolve(bindSocketToPort(this.socket))
    })
  }

  async stopServer() {
    if (this.socket) {
      await releaseSocketFromPort(this.socket)
    }
  }
}

function releaseSocketFromPort(socket: Socket) {
  return new Promise((resolve, reject) => {
    const onClose = () => {
      socket.removeListener('error', onError)
      setImmediate(resolve)
    }

    const onError = (err?: Error) => {
      socket.removeListener('close', onClose)
      reject(err)
    }

    socket.once('error', onError)

    socket.once('close', onClose)

    socket.close()
  })
}

function bindSocketToPort(socket: Socket, port = DEFAULT_STUN_PORT): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const onListening = () => {
      socket.removeListener('error', onError)
      resolve()
    }

    const onError = (err?: Error) => {
      socket.removeListener('listening', onListening)
      reject(err)
    }

    socket.once('error', onError)

    socket.once('listening', onListening)

    socket.bind(port)
  })
}
export { Stun }
