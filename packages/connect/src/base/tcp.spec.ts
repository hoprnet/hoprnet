import { createServer, type Socket, type AddressInfo } from 'net'

import { SOCKET_CLOSE_TIMEOUT, TCPConnection } from './tcp'
import { once } from 'events'
import { Multiaddr } from 'multiaddr'
import { u8aEquals, defer } from '@hoprnet/hopr-utils'
import assert from 'assert'
import type { EventEmitter } from 'events'

import { waitUntilListening, stopNode, createPeerId } from './utils.spec'

describe('test TCP connection', function () {
  it('should test TCPConnection against Node.js APIs', async function () {
    const msgReceived = defer<void>()

    const testMessage = new TextEncoder().encode('test')
    const testMessageReply = new TextEncoder().encode('reply')

    const peerId = createPeerId()

    const server = createServer((socket: Socket) => {
      socket.on('data', (data: Uint8Array) => {
        assert(u8aEquals(data, testMessage))
        socket.write(testMessageReply)

        msgReceived.resolve()
      })
    })

    await waitUntilListening<undefined | number>(server, undefined)

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

    await stopNode(server)
  })

  it('trigger a socket close timeout', async function () {
    this.timeout(SOCKET_CLOSE_TIMEOUT + 2e3)

    const testMessage = new TextEncoder().encode('test')

    const server = createServer()

    server.on('close', console.log)
    server.on('error', console.log)
    const sockets: Socket[] = []
    server.on('connection', sockets.push.bind(sockets))

    await waitUntilListening(server, undefined)

    const peerId = createPeerId()
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
    // @dev produces a half-open socket on the other side
    conn.close()

    await closePromise

    // Destroy half-open sockets.
    for (const socket of sockets) {
      socket.destroy()
    }

    await stopNode(server)

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
    const peerId = createPeerId()

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

  it('use abortController to abort streams', async function () {
    const msgReceived = defer<void>()

    const testMessage = new TextEncoder().encode('test')
    const testMessageReply = new TextEncoder().encode('reply')

    const peerId = createPeerId()

    const server = createServer((socket: Socket) => {
      socket.on('data', (data: Uint8Array) => {
        assert(u8aEquals(data, testMessage))
        socket.write(testMessageReply)

        msgReceived.resolve()
      })
    })

    await waitUntilListening<undefined | number>(server, undefined)

    const abort = new AbortController()

    const conn = await TCPConnection.create(
      new Multiaddr(`/ip4/127.0.0.1/tcp/${(server.address() as AddressInfo).port}`),
      peerId,
      {
        signal: abort.signal
      }
    )

    await assert.doesNotReject(
      async () =>
        await conn.sink(
          (async function* () {
            abort.abort()
            yield testMessage
          })()
        )
    )

    await stopNode(server)
  })
})
