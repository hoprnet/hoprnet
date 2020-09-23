import assert from 'assert'
import Listener from './listener'
import Multiaddr from 'multiaddr'
import { Connection, Upgrader } from './types'
import dgram, { Socket, RemoteInfo } from 'dgram'
import { handleStunRequest } from './stun'
import PeerId from 'peer-id'
import net from 'net'
import Defer, { DeferredPromise } from 'p-defer'

describe('check listening to sockets', function () {
  async function startStunServer(port: number, state: { msgReceived: DeferredPromise<void> }): Promise<Socket> {
    const promises: Promise<void>[] = []
    const socket = dgram.createSocket('udp4')

    promises.push(new Promise((resolve) => socket.once('listening', resolve)))

    promises.push(new Promise((resolve) => socket.bind(port, resolve)))

    socket.on('message', (msg: Buffer, rinfo: RemoteInfo) => {
      state.msgReceived.resolve()
      handleStunRequest(socket, msg, rinfo)
    })

    await Promise.all(promises)
    return socket
  }
  it('should successfully recreate the socket', async function () {
    // let listener: Listener
    // const peerId = await PeerId.create({ keyType: 'secp256k1' })
    // // Create objects to pass boolean by reference and NOT by value
    // const msgReceived = [
    //   {
    //     msgReceived: Defer<void>(),
    //   },
    //   {
    //     msgReceived: Defer<void>(),
    //   },
    // ]
    // const stunServers = [await startStunServer(9091, msgReceived[0]), await startStunServer(9092, msgReceived[1])]
    // for (let i = 0; i < 2; i++) {
    //   listener = new Listener(() => {}, (undefined as unknown) as Upgrader, [
    //     Multiaddr(`/ip4/127.0.0.1/udp/${stunServers[0].address().port}`),
    //     Multiaddr(`/ip4/127.0.0.1/udp/${stunServers[1].address().port}`),
    //   ])
    //   await listener.listen(Multiaddr(`/ip4/127.0.0.1/tcp/9090/p2p/${peerId.toB58String()}`))
    //   await listener.close()
    // }
    // await Promise.all(msgReceived.map((received) => received.msgReceived.promise))
    // stunServers.forEach((server) => server.close())
    // assert(
    //   msgReceived[0].msgReceived && msgReceived[1].msgReceived,
    //   `Stun Server must have received messages from both Listener instances.`
    // )
  })

  it('should create two servers and exchange messages', async function () {
    const AMOUNT_OF_NODES = 2

    const msgReceived = Array.from({ length: AMOUNT_OF_NODES }).map((_) => ({
      received: Defer(),
    }))

    const listeners = await Promise.all(
      Array.from({ length: AMOUNT_OF_NODES }).map(async (_, index) => {
        const peerId = await PeerId.create({ keyType: 'secp256k1' })

        const stunServers = []
        for (let i = 0; i < AMOUNT_OF_NODES; i++) {
          stunServers.push(Multiaddr(`/ip4/127.0.0.1/udp/${9090 + i}`))
        }

        const listener = new Listener(
          (conn: Connection) => {
            msgReceived[index].received.resolve()
          },
          ({
            upgradeInbound: async (conn) => conn,
          } as unknown) as Upgrader,
          stunServers
        )

        await listener.listen(Multiaddr(`/ip4/127.0.0.1/tcp/${9090 + index}/p2p/${peerId.toB58String()}`))

        return listener
      })
    )

    await Promise.all([
      new Promise((resolve) => {
        const socket = net.createConnection(
          {
            host: '127.0.0.1',
            port: 9090,
          },
          () => {
            socket.write(Buffer.from('test'), () => {
              socket.destroy()
              resolve()
            })
          }
        )
      }),
      new Promise((resolve) => {
        const socket = net.createConnection(
          {
            host: '127.0.0.1',
            port: 9091,
          },
          () => {
            socket.write(Buffer.from('test'), () => {
              socket.destroy()
              resolve()
            })
          }
        )
      }),
    ])

    await Promise.all(msgReceived.map((received) => received.received.promise))

    await Promise.all(listeners.map((listener) => listener.close()))

    await new Promise((resolve) => setTimeout(resolve, 200))

    assert(
      msgReceived.every((msg) => msg.received),
      'Should receive all messages'
    )
  })
})
