import assert from 'assert'
import Listener from './listener'
import Multiaddr from 'multiaddr'
import { Upgrader } from 'libp2p'
import type { Connection } from 'libp2p'
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
    let listener: Listener
    const peerId = await PeerId.create({ keyType: 'secp256k1' })
    // Create objects to pass boolean by reference and NOT by value
    const msgReceived = [
      {
        msgReceived: Defer<void>()
      },
      {
        msgReceived: Defer<void>()
      }
    ]

    const stunServers = [await startStunServer(9391, msgReceived[0]), await startStunServer(9392, msgReceived[1])]

    for (let i = 0; i < 2; i++) {
      listener = new Listener(() => {}, (undefined as unknown) as Upgrader, [
        Multiaddr(`/ip4/127.0.0.1/udp/${stunServers[0].address().port}`),
        Multiaddr(`/ip4/127.0.0.1/udp/${stunServers[1].address().port}`)
      ])
      await listener.listen(Multiaddr(`/ip4/127.0.0.1/tcp/9390/p2p/${peerId.toB58String()}`))
      await listener.close()
    }

    await Promise.all(msgReceived.map((received) => received.msgReceived.promise))

    await Promise.all(stunServers.map((s) => new Promise((resolve) => s.close(resolve))))

    await new Promise((resolve) => setTimeout(resolve, 200))
    assert(
      msgReceived[0].msgReceived && msgReceived[1].msgReceived,
      `Stun Server must have received messages from both Listener instances.`
    )
  })

  it('should create two TCP sockets and exchange messages', async function () {
    const AMOUNT_OF_NODES = 2

    const ATTEMPTS = 5

    let msgReceived: { received: DeferredPromise<void> }[]

    const listeners = await Promise.all(
      Array.from({ length: AMOUNT_OF_NODES }).map(async (_, index) => {
        const peerId = await PeerId.create({ keyType: 'secp256k1' })

        const stunServers = []
        for (let i = 0; i < AMOUNT_OF_NODES; i++) {
          stunServers.push(Multiaddr(`/ip4/127.0.0.1/udp/${9390 + i}`))
        }

        const listener = new Listener(
          (conn: Connection) => {
            // @ts-ignore
            conn.conn.end()
            msgReceived[index].received.resolve()
          },
          ({
            upgradeInbound: async (conn) => conn
          } as unknown) as Upgrader,
          stunServers
        )

        await listener.listen(Multiaddr(`/ip6/::/tcp/${9390 + index}/p2p/${peerId.toB58String()}`))

        return listener
      })
    )

    for (let i = 0; i < ATTEMPTS; i++) {
      msgReceived = Array.from({ length: AMOUNT_OF_NODES }).map((_) => ({
        received: Defer()
      }))

      await Promise.all([
        new Promise((resolve) => {
          const socket = net.createConnection(
            {
              host: '127.0.0.1',
              port: 9390 + (i % 2)
            },
            () => {
              socket.write(Buffer.from('test'), () => {
                socket.end()
                resolve()
              })
            }
          )
        }),
        new Promise((resolve) => {
          const socket = net.createConnection(
            {
              host: '::1',
              port: 9390 + ((i + 1) % 2)
            },
            () => {
              socket.write(Buffer.from('test'), () => {
                socket.end()
                resolve()
              })
            }
          )
        }),
        Promise.all(msgReceived.map((received) => received.received.promise))
      ])
    }

    await Promise.all(listeners.map((listener) => listener.close()))

    await new Promise((resolve) => setTimeout(resolve, 200))
  })
})
