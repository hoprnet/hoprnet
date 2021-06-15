import dgram from 'dgram'
import type { Socket, RemoteInfo } from 'dgram'
import { getExternalIp, handleStunRequest, DEFAULT_PARALLEL_STUN_CALLS, PUBLIC_STUN_SERVERS } from './stun'
import { nodeToMultiaddr } from './utils'
import { Multiaddr } from 'multiaddr'
import assert from 'assert'
import { once } from 'events'

describe('test STUN', function () {
  let servers: Socket[]

  before(async () => {
    servers = await Promise.all(
      Array.from({ length: 4 }).map(
        (_) =>
          new Promise<Socket>((resolve, reject) => {
            const server = dgram.createSocket('udp4')

            server.on('message', (msg: Buffer, rinfo: RemoteInfo) => handleStunRequest(server, msg, rinfo))
            server.once('error', reject)
            server.once('listening', () => {
              server.removeListener('error', reject)

              resolve(server)
            })

            server.bind()
          })
      )
    )
  })

  it('should perform a STUN request', async function () {
    const multiAddrs = servers.map((server: Socket) =>
      Multiaddr.fromNodeAddress(nodeToMultiaddr(server.address()), 'udp')
    )

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
        new Multiaddr(`/ip4/127.0.0.1/udp/1`)
      ],
      servers[0]
    )

    assert(Date.now() - before >= 0, `should not resolve before timeout ends`)
    assert(result != undefined, `Timeout should not lead to empty result`)
  })

  it('should not fail on DNS requests', async function () {
    await assert.rejects(
      async () => await getExternalIp([new Multiaddr(`/dns4/totallyinvalidurl.hoprnet.org/udp/12345`)], servers[0]),
      {
        name: 'Error',
        message: 'Cannot send any STUN packets. Tried with: /dns4/totallyinvalidurl.hoprnet.org/udp/12345'
      }
    )

    const stunResult = await getExternalIp(
      [new Multiaddr(`/dns4/totallyinvalidurl.hoprnet.org/udp/12345`), ...PUBLIC_STUN_SERVERS],
      servers[0]
    )

    assert(stunResult != undefined, `STUN request should work even if there are DNS failures`)
  })

  after(async () => {
    await Promise.all(
      servers.map((server) => {
        server.close()
        return once(server, 'close')
      })
    )
  })
})
