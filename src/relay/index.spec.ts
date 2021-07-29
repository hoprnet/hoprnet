/// <reference path="../@types/it-pair.ts" />
/// <reference path="../@types/it-handshake.ts" />

import { Relay } from './index'
import PeerId from 'peer-id'
import { HandlerProps } from 'libp2p'
import Pair from 'it-pair'
import EventEmitter from 'events'
import { privKeyToPeerId, stringToU8a, u8aEquals } from '@hoprnet/hopr-utils'
import handshake from 'it-handshake'
import { RelayConnection } from './connection'
import assert from 'assert'
import type { Stream, StreamType } from '../types'

const initiator = privKeyToPeerId(stringToU8a('0xa889bad3e2a31cceff4faccdd374af67db485ac0e05e7e654530aff0da5199f7'))
const relay = privKeyToPeerId(stringToU8a('0xcd1fb76053833d9bb5b3ff243b2d17b96dc5ad7cc09b33c4cf77ba83c297443f'))
const counterparty = privKeyToPeerId(stringToU8a('0x4090ca3740b1fe0f6da22befc4f7cba26389c51808d245dd29a2076fc66103aa'))

function msgToEchoedMessage(message: string) {
  return new TextEncoder().encode(`Echo: <${message}>`)
}

describe('test relay', function () {
  const connEvents = new EventEmitter()

  async function dialHelper(
    source: PeerId,
    peer: PeerId,
    protocol: string,
    _opts: any
  ): Promise<Omit<HandlerProps, 'connection'>> {
    let sourceToPeer: Stream
    let peerToSource: Stream

    const [connA, connB] = [Pair<StreamType>(), Pair<StreamType>()]
    sourceToPeer = {
      source: connB.source,
      sink: connA.sink
    }

    peerToSource = {
      source: connA.source,
      sink: connB.sink
    }

    connEvents.emit(`${peer.toB58String()}${protocol}`, {
      stream: peerToSource,
      connection: {
        remotePeer: source
      }
    } as any)

    return {
      connection: {
        remotePeer: relay
      },
      protocol,
      stream: sourceToPeer
    } as any
  }

  function handle(peer: PeerId, protocol: string, handler: (conn: HandlerProps) => void) {
    connEvents.on(`${peer.toB58String()}${protocol}`, handler)
  }

  function createPeer(source: PeerId) {
    return new Relay(
      (peer: PeerId, protocol: string, opts: any) => dialHelper(source, peer, protocol, opts) as any,
      {} as any,
      {} as any,
      (protocol: string, handler: (handler: HandlerProps) => void) => handle(source, protocol, handler),
      source,
      {
        upgradeInbound: async (conn: RelayConnection) => {
          const shaker = handshake(conn)

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

  afterEach(function () {
    connEvents.removeAllListeners()
  })

  it('connect to a relay, close the connection and reconnect', async function () {
    const Alice = createPeer(initiator)

    const Bob = createPeer(relay)

    const Charly = createPeer(counterparty)

    for (let i = 0; i < 5; i++) {
      const conn = await Alice.connect(Bob.peerId, Charly.peerId)

      assert(conn != undefined, `Should be able to connect`)
      const shaker = handshake(conn)

      const msg = '<Hello>, that should be sent and echoed through relayed connection'
      shaker.write(new TextEncoder().encode(msg))

      assert(u8aEquals((await shaker.read()).slice(), msgToEchoedMessage(msg)))

      shaker.rest()

      await conn.close()

      // Let I/O happen
      await new Promise((resolve) => setTimeout(resolve))
    }
  })

  it('connect to a relay and reconnect', async function () {
    const Alice = createPeer(initiator)

    const Bob = createPeer(relay)

    const Charly = createPeer(counterparty)

    for (let i = 0; i < 3; i++) {
      const conn = await Alice.connect(Bob.peerId, Charly.peerId)

      assert(conn != undefined, `Should be able to connect`)
      const shaker = handshake(conn)

      const msg = '<Hello>, that should be sent and echoed through relayed connection'
      shaker.write(new TextEncoder().encode(msg))

      assert(u8aEquals((await shaker.read()).slice(), msgToEchoedMessage(msg)))

      shaker.rest()

      // Let I/O happen
      await new Promise((resolve) => setTimeout(resolve))
    }
  })
})
