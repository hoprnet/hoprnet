/// <reference path="./@types/libp2p.ts" />
/// <reference path="./@types/libp2p-interfaces.ts" />

import { createServer, Socket } from 'net'
import { TCPConnection } from './tcp'
import Listener from './listener'
import Defer from 'p-defer'
import Multiaddr from 'multiaddr'
import { u8aEquals } from '@hoprnet/hopr-utils'
import type { Upgrader } from 'libp2p'
import PeerId from 'peer-id'
import { Connection } from 'libp2p-interfaces'
import assert from 'assert'

describe('test TCP connection', function () {
  it('should test TCPConnection against Node.js APIs', async function () {
    const msgReceived = Defer<void>()
    const bound = Defer<void>()

    const test = new TextEncoder().encode('test')

    const peerId = await PeerId.create({ keyType: 'secp256k1' })

    const server = createServer((socket: Socket) => {
      socket.on('data', (data: Uint8Array) => {
        assert(u8aEquals(data, test))
        msgReceived.resolve()
      })
    })

    server.listen(9091, () => {
      bound.resolve()
    })

    await bound.promise

    const conn = await TCPConnection.create(Multiaddr('/ip4/127.0.0.1/tcp/9091'), peerId)

    conn.sink(
      (async function* () {
        yield new TextEncoder().encode('test')
      })()
    )

    await new Promise<void>((resolve) => setTimeout(resolve, 50))

    await conn.close()

    await new Promise((resolve) => server.close(resolve))

    assert(conn.conn.destroyed)

    await msgReceived.promise
  })

  it('should test TCPConnection against Listener', async function () {
    const upgrader = ({
      upgradeInbound: (arg: any) => Promise.resolve(arg),
      upgradeOutbound: (arg: any) => Promise.resolve(arg)
    } as unknown) as Upgrader

    const peerId = await PeerId.create({ keyType: 'secp256k1' })
    const listener = new Listener(
      (_conn: Connection) => {
        console.log('new connection')
      },
      upgrader,
      undefined,
      undefined,
      peerId,
      undefined
    )

    await listener.listen(Multiaddr('/ip4/127.0.0.1/tcp/9091'))

    const tcpConn = await TCPConnection.create(Multiaddr('/ip4/127.0.0.1/tcp/9091'), peerId)

    tcpConn.sink(
      (async function* () {
        yield new TextEncoder().encode('test')
      })()
    )

    await tcpConn.close()

    await listener.close()

    await new Promise((resolve) => setTimeout(resolve, 100))
  })
})
