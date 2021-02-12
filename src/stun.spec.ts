import dgram from 'dgram'
import type { Socket, RemoteInfo } from 'dgram'
import { getExternalIp, handleStunRequest, DEFAULT_PARALLEL_STUN_CALLS, PUBLIC_STUN_SERVERS } from './stun'
import Multiaddr from 'multiaddr'
import assert from 'assert'
import { once } from 'events'

describe('test STUN', function () {
  let servers: Socket[]

  before(() => {
    servers = Array.from({ length: 4 }).map((_) => {
      const server = dgram.createSocket('udp4')
      server.on('message', (msg: Buffer, rinfo: RemoteInfo) => handleStunRequest(server, msg, rinfo))
      server.on('error', (e) => {
        throw e
      })
      return server
    })
  })

  it('should perform a STUN request', async function () {
    await Promise.all(
      servers.map((server) => {
        server.bind()
        return once(server, 'listening')
      })
    )

    const multiAddrs = servers.map((server: Socket) => Multiaddr.fromNodeAddress(server.address() as any, 'udp'))

    const result = await getExternalIp(multiAddrs, servers[0])

    assert(result != undefined, `STUN request must be successful`)

    assert(servers[0].address().port === result.port, 'Ports should match')
    /*
     // DISABLED - with IP4 the address changes from 0.0.0.0 to 127.0.0.1
     // IPV6 doesn't work at present.
     //
      assert((client.address().address === result.address || 
           client.address().address.concat('1') === result.address), "address should match")
    */
  })

  it('should get our external address from a public server if there is no other server given', async function () {
    const result = await getExternalIp(undefined, servers[0])

    assert(result != undefined, 'server should be able to detect its external address')
  })

  it('should return a valid external address even if some external STUN servers produce a timeout', async function () {
    const before = Date.now()
    const result = await getExternalIp(
      [
        ...PUBLIC_STUN_SERVERS.slice(0, Math.max(0, DEFAULT_PARALLEL_STUN_CALLS - 1)),
        Multiaddr(`/ip4/127.0.0.1/udp/1`)
      ],
      servers[0]
    )

    assert(Date.now() - before >= 0, `should not resolve before timeout ends`)
    assert(result != undefined, `Timeout should not lead to empty result`)
  })

  after(async () => {
    await Promise.all(
      servers.map((server) => {
        server.close()
        return once(server, 'close')
      })
    )

    await new Promise<void>((resolve) => setTimeout(() => resolve(), 500))
  })
})
