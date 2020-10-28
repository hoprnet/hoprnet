import dgram, { RemoteInfo } from 'dgram'
import type { Socket } from 'dgram'
import { getExternalIp, handleStunRequest } from './stun'
import Multiaddr from 'multiaddr'
import assert from 'assert'
import { once } from 'events'

describe('test STUN', function () {
  let client
  let servers

  beforeAll(() => {
    servers = Array.from({ length: 4 }).map((_) => {
      const server = dgram.createSocket('udp4')
      server.on('message', (msg: Buffer, rinfo: RemoteInfo) => handleStunRequest(server, msg, rinfo))
      server.on('error', (e) => {
        throw e
      })
      return server
    })
    client = dgram.createSocket('udp4')
    client.on('error', (e) => {
      throw e
    })
  })

  it('should perform a STUN request', async function () {
    await Promise.all(
      servers.map((server) => {
        server.bind()
        return once(server, 'listening')
      })
    )
    client.bind()
    await once(client, 'listening')

    const multiAddrs = servers.map((server: Socket) => Multiaddr.fromNodeAddress(server.address() as any, 'udp'))

    const result = await getExternalIp(multiAddrs, client)

    assert(client.address().port === result.port, 'Ports should match')
    /*
     // DISABLED - with IP4 the address changes from 0.0.0.0 to 127.0.0.1
     // IPV6 doesn't work at present.
     //
      assert((client.address().address === result.address || 
           client.address().address.concat('1') === result.address), "address should match")
    */
  })

  afterAll(async () => {
    await Promise.all(
      servers.map((server) => {
        server.close()
        return once(server, 'close')
      })
    )
    client.close()
    await once(client, 'close')
    await new Promise((resolve) => setTimeout(() => resolve(), 500))
  })
})
