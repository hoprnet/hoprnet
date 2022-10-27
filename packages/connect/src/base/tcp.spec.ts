import { createServer, type Socket, type AddressInfo } from 'net'
import { setTimeout as setTimeoutPromise } from 'timers/promises'

import { createTCPConnection, fromSocket } from './tcp.js'
import { Multiaddr } from '@multiformats/multiaddr'
import { u8aEquals, defer } from '@hoprnet/hopr-utils'
import assert from 'assert'

import { waitUntilListening, stopNode } from './utils.spec.js'
import { Writable } from 'stream'

describe('test TCP connection', function () {
  it('should test TCPConnection against Node.js APIs', async function () {
    const msgReceived = defer<void>()

    const testMessage = new TextEncoder().encode('test')
    const testMessageReply = new TextEncoder().encode('reply')

    const server = createServer((socket: Socket) => {
      socket.on('data', (data: Uint8Array) => {
        assert(u8aEquals(data, testMessage))
        socket.write(testMessageReply)

        msgReceived.resolve()
      })
    })

    await waitUntilListening<undefined | number>(server, undefined)

    const closePromise = defer<void>()

    const conn = await createTCPConnection(
      new Multiaddr(`/ip4/127.0.0.1/tcp/${(server.address() as AddressInfo).port}`),
      () => {
        closePromise.resolve()
      }
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

    await conn.close()

    // Should trigger close function
    await closePromise.promise

    assert(
      conn.timeline.close != undefined &&
        conn.timeline.close <= Date.now() &&
        conn.timeline.open <= conn.timeline.close,
      `TCPConnection must populate timeline object`
    )

    await stopNode(server)
  })

  it('tcp socket timeout and error cases', async function () {
    const INVALID_PORT = 54221

    await assert.rejects(
      async () => {
        await createTCPConnection(new Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}`), () => {})
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

    const server = createServer((socket: Socket) => {
      socket.on('data', (data: Uint8Array) => {
        assert(u8aEquals(data, testMessage))
        socket.write(testMessageReply)

        msgReceived.resolve()
      })
    })

    await waitUntilListening<undefined | number>(server, undefined)

    const abort = new AbortController()

    const conn = await createTCPConnection(
      new Multiaddr(`/ip4/127.0.0.1/tcp/${(server.address() as AddressInfo).port}`),
      () => {},
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

describe('test TCP connection - socket errors', function () {
  it('throw on write attempts', async function () {
    const socket = new Writable()
    // Overwrite methods to simulate socket errors
    Object.assign(socket, {
      write: () => {
        throw Error(`boom`)
      },
      setKeepAlive: () => {},
      [Symbol.asyncIterator]: () => {
        return {
          next() {
            return Promise.resolve({ value: new Uint8Array(), done: false })
          },
          return() {
            // This will be reached if the consumer called 'break' or 'return' early in the loop.
            return { done: true }
          }
        }
      },
      remoteAddress: '127.0.0.1',
      remotePort: 9091,
      remoteFamily: 'IPv4',
      address: () => ({
        address: '127.0.0.1',
        port: 9092,
        family: 'IPv4'
      })
    })
    const conn = fromSocket(socket as Socket, () => {})
    await conn.sink(
      (async function* (): AsyncIterable<Uint8Array> {
        // propagation delay
        await setTimeoutPromise(200)
        yield new Uint8Array()
      })()
    )
    // Propagation delay to let errors happen
    await setTimeoutPromise(500)
  })
})
