/// <reference path="../@types/it-pair.ts" />
/// <reference path="../@types/it-handshake.ts" />
/// <reference path="../@types/libp2p.ts" />

import { Relay } from './index'
import PeerId from 'peer-id'
import { Handler, Stream, StreamType } from 'libp2p'
import Pair from 'it-pair'
import EventEmitter from 'events'
import { privKeyToPeerId, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import handshake from 'it-handshake'
import { RelayConnection } from './connection'
import assert from 'assert'

const initiator = privKeyToPeerId(stringToU8a('0xa889bad3e2a31cceff4faccdd374af67db485ac0e05e7e654530aff0da5199f7'))
console.log(`initiator`, initiator.toB58String())
const relay = privKeyToPeerId(stringToU8a('0xcd1fb76053833d9bb5b3ff243b2d17b96dc5ad7cc09b33c4cf77ba83c297443f'))
console.log(`relay`, relay.toB58String())
const counterparty = privKeyToPeerId(stringToU8a('0x4090ca3740b1fe0f6da22befc4f7cba26389c51808d245dd29a2076fc66103aa'))
console.log(`counterparty`, counterparty.toB58String())

function msgToEchoedMessage(message: string) {
  return new TextEncoder().encode(`Echo: <${message}>`)
}

describe('test relay', function () {
  const connections = new Map<string, Stream>()

  const connEvents = new EventEmitter()

  async function dialHelper(source: PeerId, peer: PeerId, protocol: string, _opts: any): Promise<Handler> {
    const aToB = `${source.toB58String()}${peer.toB58String()}${protocol}`
    const bToA = `${peer.toB58String()}${source.toB58String()}${protocol}`

    let sourceToPeer: Stream
    let peerToSource: Stream

    if (connections.has(aToB) && connections.has(bToA)) {
      sourceToPeer = connections.get(aToB) as Stream
      peerToSource = connections.get(bToA) as Stream
    } else {
      const [connA, connB] = [Pair(), Pair()]
      sourceToPeer = {
        source: connB.source,
        sink: connA.sink
      }

      peerToSource = {
        source: connA.source,
        sink: connB.sink
      }

      connections.set(aToB, sourceToPeer)
      connections.set(bToA, peerToSource)
    }

    connEvents.emit(`${peer.toB58String()}${protocol}`, {
      stream: peerToSource,
      connection: {
        remotePeer: source
      }
    } as Handler)

    return {
      connection: {
        remotePeer: relay
      },
      protocol,
      stream: sourceToPeer
    } as Handler
  }

  function handle(peer: PeerId, protocol: string, handler: (conn: Handler) => void) {
    connEvents.on(`${peer.toB58String()}${protocol}`, handler)
  }

  function createPeer(source: PeerId) {
    return new Relay(
      (peer: PeerId, protocol: string, opts: any) => dialHelper(source, peer, protocol, opts),
      {} as any,
      {} as any,
      (protocol: string, handler: (handler: Handler) => void) => handle(source, protocol, handler),
      source,
      {
        upgradeInbound: async (conn: RelayConnection) => {
          const shaker = handshake<StreamType>(conn)

          const message = new TextDecoder().decode((await shaker.read()).slice())

          shaker.write(msgToEchoedMessage(message))

          shaker.rest()
        },
        upgradeOutbound: (conn: any) => conn
      } as any,
      undefined,
      undefined,
      undefined
    )
  }

  //   after(function () {
  //       connEvents.removeAllListeners()
  //       connections.clear()
  //   })

  it('connect to a relay', async function () {
    const Alice = createPeer(initiator)

    const Bob = createPeer(relay)

    const Charly = createPeer(counterparty)

    const conn = await Alice.connect(relay, counterparty)

    assert(conn != undefined, `Should be able to connect`)
    const shaker = handshake<StreamType>(conn)

    const msg = 'foo'
    shaker.write(new TextEncoder().encode(msg))

    assert(u8aEquals((await shaker.read()).slice(), msgToEchoedMessage(msg)))

    shaker.rest()

    await conn.close()
  })
})
