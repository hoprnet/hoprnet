import { createServer, Socket } from 'net'
import type { AddressInfo } from 'net'
import { SOCKET_CLOSE_TIMEOUT, TCPConnection } from './tcp'
import Defer from 'p-defer'
import { once } from 'events'
import { Multiaddr } from 'multiaddr'
import { u8aEquals } from '@hoprnet/hopr-utils'
import PeerId from 'peer-id'
import assert from 'assert'
import type { EventEmitter } from 'events'

describe('test TCP connection', function () {
  it('should test TCPConnection against Node.js APIs', async function () {
    const msgReceived = Defer<void>()

    const testMessage = new TextEncoder().encode('test')
    const testMessageReply = new TextEncoder().encode('reply')

    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const server = createServer((socket: Socket) => {
      socket.on('data', (data: Uint8Array) => {
        assert(u8aEquals(data, testMessage))
        socket.write(testMessageReply)

        msgReceived.resolve()
      })
    })

    const boundPromise = once(server, 'listening')
    server.listen()

    await boundPromise

    const conn = await TCPConnection.create(
      new Multiaddr(`/ip4/127.0.0.1/tcp/${(server.address() as AddressInfo).port}`),
      peerId
    )

    await conn.sink(
      (async function* () {
        yield testMessage
      })()
    )

    for await (const msg of conn.source) {
      assert(u8aEquals(msg.slice(), testMessageReply))
    }

    await msgReceived.promise

    conn.close()

    await once(conn.conn as EventEmitter, 'close')

    assert(conn.conn.destroyed)

    assert(
      conn.timeline.close != undefined &&
        conn.timeline.close <= Date.now() &&
        conn.timeline.open <= conn.timeline.close,
      `TCPConnection must populate timeline object`
    )

    const serverClosePromise = once(server, 'close')
    server.close()

    await serverClosePromise
  })

  it('trigger a socket close timeout', async function () {
    this.timeout(SOCKET_CLOSE_TIMEOUT + 2e3)

    const testMessage = new TextEncoder().encode('test')

    const server = createServer()

    const boundPromise = once(server, 'listening')
    server.listen()

    await boundPromise

    const peerId = await PeerId.create({ keyType: 'secp256k1' })
    const conn = await TCPConnection.create(
      new Multiaddr(`/ip4/127.0.0.1/tcp/${(server.address() as AddressInfo).port}`),
      peerId
    )

    await conn.sink(
      (async function* () {
        yield testMessage
      })()
    )

    const start = Date.now()
    const closePromise = once(conn.conn, 'close')
    conn.close()

    await closePromise

    assert(Date.now() - start >= SOCKET_CLOSE_TIMEOUT)

    assert(
      conn.timeline.close != undefined &&
        conn.timeline.close <= Date.now() &&
        conn.timeline.open <= conn.timeline.close,
      `TCPConnection must populate timeline object`
    )
  })

  it('tcp socket timeout and error cases', async function () {
    const INVALID_PORT = 54221
    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    await assert.rejects(
      async () => {
        await TCPConnection.create(new Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}`), peerId)
      },
      {
        name: 'Error',
        code: 'ECONNREFUSED'
      }
    )
  })
})
