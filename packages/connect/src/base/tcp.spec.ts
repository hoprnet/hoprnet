// import { createServer, type Socket, type AddressInfo } from 'net'
// import { setTimeout } from 'timers/promises'

// import { createTCPConnection, fromSocket } from './tcp.js'
// import { once } from 'events'
// import { Multiaddr } from '@multiformats/multiaddr'
// import { u8aEquals, defer } from '@hoprnet/hopr-utils'
// import assert from 'assert'
// import type { EventEmitter } from 'events'

// import { waitUntilListening, stopNode } from './utils.spec.js'
// import { Writable } from 'stream'

// describe('test TCP connection', function () {
//   it('should test TCPConnection against Node.js APIs', async function () {
//     const msgReceived = defer<void>()

//     const testMessage = new TextEncoder().encode('test')
//     const testMessageReply = new TextEncoder().encode('reply')

//     const server = createServer((socket: Socket) => {
//       socket.on('data', (data: Uint8Array) => {
//         assert(u8aEquals(data, testMessage))
//         socket.write(testMessageReply)

//         msgReceived.resolve()
//       })
//     })

//     await waitUntilListening<undefined | number>(server, undefined)

//     const conn = await createTCPConnection(
//       new Multiaddr(`/ip4/127.0.0.1/tcp/${(server.address() as AddressInfo).port}`)
//     )

//     await conn.sink(
//       (async function* () {
//         yield testMessage
//       })()
//     )

//     for await (const msg of conn.source) {
//       assert(u8aEquals(msg.slice(), testMessageReply))
//     }

//     await msgReceived.promise

//     await conn.close()

//     await once(conn.socket as EventEmitter, 'close')

//     assert(conn.destroyed)

//     assert(
//       conn.timeline.close != undefined &&
//         conn.timeline.close <= Date.now() &&
//         conn.timeline.open <= conn.timeline.close,
//       `TCPConnection must populate timeline object`
//     )

//     await stopNode(server)
//   })

//   // it('trigger a socket close timeout', async function () {
//   //   const SOCKET_CLOSE_TIMEOUT = 1000

//   //   this.timeout(SOCKET_CLOSE_TIMEOUT + 2e3)

//   //   // test server with tweaked behavior
//   //   const testServer = createServer()
//   //   const testMessage = new TextEncoder().encode('test message')

//   //   const sockets: Socket[] = []
//   //   testServer.on('connection', (socket: Socket) => {
//   //     // Handle incoming data
//   //     socket.on('data', (data) => {
//   //       assert(u8aEquals(data, testMessage))
//   //     })
//   //     sockets.push(socket)
//   //   })

//   //   // make testServer listen at a random port
//   //   await waitUntilListening<Socket>(testServer, undefined as any)

//   //   const conn = await createTCPConnection(
//   //     new Multiaddr(`/ip4/127.0.0.1/tcp/${(testServer.address() as AddressInfo).port}`),
//   //     {
//   //       closeTimeout: 1000
//   //     }
//   //   )

//   //   conn.sink(
//   //     (async function* () {
//   //       while (true) {
//   //         yield testMessage
//   //         await setTimeout(10)
//   //       }
//   //     })()
//   //   )

//   //   const destroy = conn.socket.destroy.bind(conn.socket)
//   //   Object.assign(conn.socket, {
//   //     destroy: () => {}
//   //   })

//   //   const start = Date.now()
//   //   const closePromise = once(conn.socket, 'close')

//   //   // @dev produces a half-open socket on the other side
//   //   conn.close()

//   //   assert(conn.closed === true, `Connection must be marked closed`)

//   //   // Wait some time before restoring `destroy()` method
//   //   await setTimeout(SOCKET_CLOSE_TIMEOUT / 2)

//   //   Object.assign(conn.socket, {
//   //     destroy
//   //   })

//   //   // Now that `destroy()` method has been restored, await `close` event
//   //   await closePromise

//   //   // Destroy all sockets of testServer
//   //   for (const socket of sockets) {
//   //     socket.destroy()
//   //   }

//   //   await stopNode(testServer)

//   //   assert(Date.now() - start >= SOCKET_CLOSE_TIMEOUT, `should not timeout earlier than configured timeout`)

//   //   assert(
//   //     conn.timeline.close != undefined &&
//   //       conn.timeline.close <= Date.now() &&
//   //       conn.timeline.open <= conn.timeline.close,
//   //     `TCPConnection must populate timeline object`
//   //   )
//   // })

//   it('tcp socket timeout and error cases', async function () {
//     const INVALID_PORT = 54221

//     await assert.rejects(
//       async () => {
//         await createTCPConnection(new Multiaddr(`/ip4/127.0.0.1/tcp/${INVALID_PORT}`))
//       },
//       {
//         name: 'Error',
//         code: 'ECONNREFUSED'
//       }
//     )
//   })

//   it('use abortController to abort streams', async function () {
//     const msgReceived = defer<void>()

//     const testMessage = new TextEncoder().encode('test')
//     const testMessageReply = new TextEncoder().encode('reply')

//     const server = createServer((socket: Socket) => {
//       socket.on('data', (data: Uint8Array) => {
//         assert(u8aEquals(data, testMessage))
//         socket.write(testMessageReply)

//         msgReceived.resolve()
//       })
//     })

//     await waitUntilListening<undefined | number>(server, undefined)

//     const abort = new AbortController()

//     const conn = await createTCPConnection(
//       new Multiaddr(`/ip4/127.0.0.1/tcp/${(server.address() as AddressInfo).port}`),
//       {
//         signal: abort.signal
//       }
//     )

//     await assert.doesNotReject(
//       async () =>
//         await conn.sink(
//           (async function* () {
//             abort.abort()
//             yield testMessage
//           })()
//         )
//     )

//     await stopNode(server)
//   })
// })

// describe('test TCP connection - socket errors', function () {
//   it('throw on write attempts', async function () {
//     const socket = new Writable()

//     // Overwrite methods to simulate socket errors
//     Object.assign(socket, {
//       write: () => {
//         throw Error(`boom`)
//       },
//       [Symbol.asyncIterator]: () => {
//         return {
//           next() {
//             return Promise.resolve({ value: new Uint8Array(), done: false })
//           },
//           return() {
//             // This will be reached if the consumer called 'break' or 'return' early in the loop.
//             return { done: true }
//           }
//         }
//       },
//       remoteAddress: '127.0.0.1',
//       remotePort: 9091,
//       remoteFamily: 'IPv4',
//       address: () => ({
//         address: '127.0.0.1',
//         port: 9092,
//         family: 'IPv4'
//       })
//     })

//     const conn = fromSocket(socket as Socket)

//     await conn.sink(
//       (async function* (): AsyncIterable<Uint8Array> {
//         // propagation delay
//         await setTimeout(200)
//         yield new Uint8Array()
//       })()
//     )

//     // Propagation delay to let errors happen
//     await setTimeout(500)
//   })
// })
