import type { StreamType } from '../types'
import type { HandlerProps } from 'libp2p'
import type Connection from 'libp2p-interfaces/src/connection/connection'
import type { Address } from 'libp2p/src/peer-store/address-book'

import { Relay } from './index'
import PeerId from 'peer-id'
import EventEmitter from 'events'
import { privKeyToPeerId, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import handshake from 'it-handshake'
import { RelayConnection } from './connection'
import assert from 'assert'
import Pair from 'it-pair'
import { Multiaddr } from 'multiaddr'

const initiator = privKeyToPeerId(stringToU8a('0xa889bad3e2a31cceff4faccdd374af67db485ac0e05e7e654530aff0da5199f7'))
const relay = privKeyToPeerId(stringToU8a('0xcd1fb76053833d9bb5b3ff243b2d17b96dc5ad7cc09b33c4cf77ba83c297443f'))
const counterparty = privKeyToPeerId(stringToU8a('0x4090ca3740b1fe0f6da22befc4f7cba26389c51808d245dd29a2076fc66103aa'))

function msgToEchoedMessage(message: string): Uint8Array {
  return new TextEncoder().encode(`Echo: <${message}>`)
}

function getPeerProtocol(peer: PeerId, protocol: string) {
  return `${peer.toB58String()}${protocol}`
}

function getPeer(peerId: PeerId, network: EventEmitter) {
  async function handle(protocol: string, handler: (conn: HandlerProps) => void) {
    network.on(getPeerProtocol(peerId, protocol), handler)
  }

  async function dialDirectly(ma: Multiaddr): Promise<Connection> {
    const peerId = PeerId.createFromB58String(ma.getPeerId() as string)

    return {
      remotePeer: peerId,
      newStream: async (protocol: string) => {
        const AtoB = Pair<StreamType>()
        const BtoA = Pair<StreamType>()

        network.emit(getPeerProtocol(peerId, protocol), {
          stream: {
            sink: AtoB.sink,
            source: BtoA.source
          },
          connection: {
            remotePeer: peerId
          }
        })

        return {
          protocol,
          stream: {
            sink: BtoA.sink,
            source: AtoB.source
          }
        }
      }
    } as any
  }

  return new Relay(
    {
      peerId,
      handle,
      upgrader: {
        upgradeInbound: (async (conn: RelayConnection) => {
          const shaker = handshake<StreamType>(conn as any)

          const message = new TextDecoder().decode((await shaker.read()).slice())

          shaker.write(msgToEchoedMessage(message))

          shaker.rest()
        }) as any,
        upgradeOutbound: (conn: any) => conn
      },
      peerStore: {
        addressBook: {
        get: async (peer: PeerId): Promise<Address[]> => {
          return [
              {
                multiaddr: new Multiaddr(`/ip4/127.0.0.1/tcp/1/p2p/${peer.toB58String()}`),
                isCertified: true
              }
            ]
        }
        }
      },
      dialer: {} as any,
      connectionManager: {
        get: () => null
      } as any,
      contentRouting: {
        provide: (_key: any) => Promise.resolve()
      }
    },
    dialDirectly,
    (multiaddrs: Multiaddr[]) => multiaddrs,
    { environment: `testingEnvironment` },
    { __noWebRTCUpgrade: true }
  )
}

describe('test relay', function () {
  it('connect to a relay, close the connection and reconnect', async function () {
    const network = new EventEmitter()

    const Alice = getPeer(initiator, network)
    const Bob = getPeer(relay, network)
    const Charly = getPeer(counterparty, network)

    Alice.start()
    Bob.start()
    Charly.start()

    for (let i = 0; i < 5; i++) {
      const conn = await Alice.connect(Bob.libp2p.peerId, Charly.libp2p.peerId)

      assert(conn != undefined, `Should be able to connect`)
      const shaker = handshake<StreamType>(conn as any)

      const msg = '<Hello>, that should be sent and echoed through relayed connection'
      shaker.write(new TextEncoder().encode(msg))

      assert(u8aEquals((await shaker.read()).slice(), msgToEchoedMessage(msg)))

      shaker.rest()

      await conn.close()

      // Let I/O happen
      await new Promise((resolve) => setTimeout(resolve))
    }

    network.removeAllListeners()
  })

  it('connect to a relay and reconnect', async function () {
    const network = new EventEmitter()

    const Alice = getPeer(initiator, network)
    const Bob = getPeer(relay, network)
    const Charly = getPeer(counterparty, network)

    Alice.start()
    Bob.start()
    Charly.start()

    for (let i = 0; i < 3; i++) {
      const conn = await Alice.connect(Bob.libp2p.peerId, Charly.libp2p.peerId)

      assert(conn != undefined, `Should be able to connect`)
      const shaker = handshake<StreamType>(conn as any)

      const msg = '<Hello>, that should be sent and echoed through relayed connection'
      shaker.write(new TextEncoder().encode(msg))

      assert(u8aEquals((await shaker.read()).slice(), msgToEchoedMessage(msg)))

      shaker.rest()

      // Let I/O happen
      await new Promise((resolve) => setTimeout(resolve))
    }

    network.removeAllListeners()
  })
})
