import dgram, { RemoteInfo } from 'dgram'
import type { Socket } from 'dgram'
import { getExternalIp, handleStunRequest } from './stun'
import Multiaddr from 'multiaddr'
import assert from 'assert'

describe('test STUN', function () {
  it('should perform a STUN request', async function () {
    const bindPromises: Promise<void>[] = []
    const servers = Array.from({ length: 4 }).map((_) => {
      const server = dgram.createSocket('udp6')

      bindPromises.push(
        new Promise<void>((resolve) => server.once('listening', resolve))
      )
      server.bind()

      server.on('message', (msg: Buffer, rinfo: RemoteInfo) => handleStunRequest(server, msg, rinfo))

      return server
    })

    const client = dgram.createSocket('udp6')

    bindPromises.push(
      new Promise<void>((resolve) => client.once('listening', resolve))
    )

    client.bind()

    await Promise.all(bindPromises)

    const multiAddrs = servers.map((server: Socket) => Multiaddr.fromNodeAddress(server.address() as any, 'udp'))

    const result = await getExternalIp(multiAddrs, client)

    assert(
      client.address().port == result.port &&
        (client.address().address === result.address || client.address().address.concat('1') === result.address)
    )

    servers.forEach((server) => server.close())
    client.close()

    await new Promise((resolve) => setTimeout(() => resolve(), 140))
  })
})
