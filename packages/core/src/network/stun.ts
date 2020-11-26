import dgram, { Socket } from 'dgram'
import * as stun from 'webrtc-stun'
import { HoprOptions } from '..'

import { DEFAULT_STUN_PORT } from '../constants'

import { durations } from '@hoprnet/hopr-utils'

export type Interface = {
  family: 'IPv4' | 'IPv6'
  port: number
  address: string
}

const STUN_TIMEOUT = durations.seconds(4)

class Stun {
  private socket: Socket

  constructor(private hosts: HoprOptions['hosts']) {}

  static getExternalIP(addresses: { hostname: string; port: number }[], usePort?: number): Promise<Interface> {
    return new Promise<Interface>(async (resolve, reject) => {
      let attr: Interface

      const timeout = setTimeout(async () => {
        await releaseSocketFromPort(socket)

        if (attr != null) {
          resolve(attr)
        } else {
          reject(new Error(`Timeout during STUN request.`))
        }
      }, STUN_TIMEOUT)
      const socket = dgram.createSocket({ type: 'udp4' })
      const tids: string[] = []

      for (let i = 0; i < addresses.length; i++) {
        tids.push(stun.generateTransactionId())
      }

      if (usePort !== undefined) {
        await bindSocketToPort(socket, usePort)
      }

      const onMessage = async (msg: Buffer) => {
        const res = stun.createBlank()

        const logBackup = console.log
        console.log = () => {}
        if (res.loadBuffer(msg)) {
          if (
            tids.some((tid, index, array) => {
              if (res.isBindingResponseSuccess({ transactionId: tid, fingerprint: true })) {
                array.splice(index, 1)
                return true
              }
              return false
            })
          ) {
            const receivedAttr = res.getXorMappedAddressAttribute() as Interface

            if (receivedAttr) {
              if (attr == null) {
                attr = receivedAttr
              } else if (attr.port != receivedAttr.port) {
                attr.port = undefined
              }

              if (tids.length == 0) {
                socket.removeListener('message', onMessage)
                await releaseSocketFromPort(socket)
                clearTimeout(timeout)
                resolve(attr)
              }
            }
          }
        }
        console.log = logBackup
      }

      socket.on('message', onMessage)

      for (let i = 0; i < addresses.length; i++) {
        const req = stun.createBindingRequest(tids[i]).setFingerprintAttribute()

        socket.send(req.toBuffer(), addresses[i].port, addresses[i].hostname)
      }
    })
  }

  getSocket() {
    if (this.hosts === undefined) {
      return dgram.createSocket({ type: 'udp4' })
    }
    
    if (this.hosts.ip4 !== undefined && this.hosts.ip6 !== undefined) {
      return dgram.createSocket({ type: 'udp6' })
    } else if (this.hosts.ip4 !== undefined) {
      return dgram.createSocket({ type: 'udp4' })
    } else if (this.hosts.ip6 !== undefined) {
      return dgram.createSocket({ type: 'udp6', ipv6Only: true })
    }
    throw Error(`Cannot create STUN socket due to invalid configuration.`)
  }

  async startServer(port?: number) {
    return new Promise(async (resolve) => {
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

      resolve(bindSocketToPort(this.socket, port))
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
      resolve()
    }

    const onError = (err?: Error) => {
      socket.removeAllListeners()
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
export default Stun
