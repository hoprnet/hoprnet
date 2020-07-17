import { EventEmitter } from 'events'
import { Injectable } from '@nestjs/common'
import multiaddr from 'multiaddr'
import PeerId from 'peer-id'
import PeerInfo from 'peer-info'
import type { HoprOptions } from '@hoprnet/hopr-core'
import { IsPort, IsIP } from 'class-validator'
import { default as parseIpPort } from 'parse-ip-port'
import { isIPv4 } from 'net'
import { decode } from 'rlp'

type ParserError = {
  readonly message: string
}

type Message = {
  message: string
  latency: number
}

class Host {
  @IsIP(4)
  ipv4: string

  @IsIP(6)
  ipv6: string

  @IsPort()
  port: number

  private _hosts: HoprOptions['hosts'] = {}

  get hosts(): HoprOptions['hosts'] {
    this._hosts.ip4 = { ip: this.ipv4, port: this.port }
    // @TODO: Restore ipv6 support and/or condition it
    // this._hosts.ip6 = { ip: this.ipv6, port: this.port }
    return this._hosts
  }
}

@Injectable()
export class ParserService {
  async parseBootstrap(bootstrapServer: string): Promise<ParserError | PeerInfo> {
    const bootstrapMultiAddress = multiaddr(bootstrapServer.trim())
    const peerId = bootstrapMultiAddress.getPeerId()
    const translatedPeerId = PeerId.createFromB58String(peerId)
    const peerInfoCreationResponse = await PeerInfo.create(translatedPeerId).catch((err) => {
      return { message: err }
    })
    if (peerInfoCreationResponse instanceof PeerInfo) {
      peerInfoCreationResponse.multiaddrs.add(bootstrapMultiAddress)
    }
    return peerInfoCreationResponse
  }

  parseHost(host: string): Promise<ParserError | HoprOptions['hosts']> {
    return new Promise((resolve, reject) => {
      try {
        const [ip, port] = parseIpPort(host)
        const hostObject = new Host()
        const { ipType, ipValue } = isIPv4(ip) ? { ipType: 'ipv4', ipValue: ip } : { ipType: 'ipv6', ipValue: ip }
        hostObject[ipType] = ipValue
        hostObject.port = port
        resolve(hostObject.hosts)
      } catch (err) {
        return reject({ message: err })
      }
    })
  }

  outputFunctor(events: EventEmitter): (encoded: Uint8Array) => Message {
    return (encoded: Uint8Array): Message => {
      const [messageBuffer, latencyBuffer] = decode(encoded) as [Buffer, Buffer]
      const message = messageBuffer.toString()
      const latency = Date.now() - parseInt(latencyBuffer.toString('hex'), 16)

      events.emit('message', messageBuffer)

      return { message, latency }
    }
  }
}
