import dgram from 'dgram'
import type { Socket, RemoteInfo } from 'dgram'
import {
  getExternalIp,
  handleStunRequest,
  DEFAULT_PARALLEL_STUN_CALLS,
  PUBLIC_STUN_SERVERS,
  STUN_TIMEOUT
} from './stun'
import { ipToU8aAddress, isLocalhost, isPrivateAddress, nodeToMultiaddr } from '../utils'
import { Multiaddr } from 'multiaddr'
import assert from 'assert'
import { once } from 'events'
import { defer } from '@hoprnet/hopr-utils'
import type { DeferType } from '@hoprnet/hopr-utils'

type ServerType = {
  socket: Socket
  gotContacted: DeferType<number>
  contactCount: number
  index: number
}

/**
 * Creates a STUN server that answers with tweaked STUN responses to simulate
 * ambiguous results from STUN servers
 * @param port fake port
 * @param address fake address
 * @returns STUN server answering with falsy responses
 */
async function getAmbiguousSTUNServer(port: number, address?: string) {
  const socket = dgram.createSocket('udp4')

  const listeningPromise = once(socket, 'listening')
  socket.bind()

  socket.on('message', (msg: Buffer, rinfo: RemoteInfo) => {
    handleStunRequest(socket, msg, rinfo, {
      ...rinfo,
      address: address ?? rinfo.address,
      port
    })
  })

  await listeningPromise

  return socket
}

function closeSTUNServer(socket: Socket) {
  const closePromise = once(socket, 'close')

  socket.close()

  return closePromise
}

describe('test STUN', function () {
  let servers: ServerType[]

  before(async () => {
    servers = await Promise.all(
      // 1 STUN server that contacts
      // DEFAULT_PARALLEL_STUN_CALLS STUN servers and leaves out
      // 1 available STUN server
      Array.from({ length: DEFAULT_PARALLEL_STUN_CALLS + 2 }).map(
        (_: any, index: number) =>
          new Promise<ServerType>((resolve, reject) => {
            const socket = dgram.createSocket('udp4')

            const gotContacted = defer<number>()
            let contactCount = 0

            socket.on('message', (msg: Buffer, rinfo: RemoteInfo) => {
              gotContacted.resolve(index)
              contactCount++
              handleStunRequest(socket, msg, rinfo)
            })
            socket.once('error', reject)
            socket.once('listening', () => {
              socket.removeListener('error', reject)

              resolve({
                socket,
                gotContacted,
                contactCount,
                index
              })
            })

            socket.bind()
          })
      )
    )
  })

  it('should perform a STUN request', async function () {
    const multiAddrs = servers
      .slice(1)
      .map((server: ServerType) => Multiaddr.fromNodeAddress(nodeToMultiaddr(server.socket.address()), 'udp'))

    let result = await getExternalIp(multiAddrs, servers[0].socket)

    if (result == undefined) {
      // It seems that we're running behind a bidirectional NAT. Test STUN in local mode
      result = await getExternalIp(multiAddrs, servers[0].socket, true)

      assert(result != undefined, `STUN request must be successful`)

      assert(isLocalhost(ipToU8aAddress(result.address, 'IPv4'), 'IPv4'))
    } else {
      // NAT seems to be honest or running on a machine with public IPv4 address
      const u8aAddress = ipToU8aAddress(result.address, 'IPv4')

      // Check that returned is a public address
      assert(!isLocalhost(u8aAddress, 'IPv4'), `Returned address must not be localhost`)

      assert(!isPrivateAddress(u8aAddress, 'IPv4'), `Returned address must not be a local address`)
    }

    /*
     // DISABLED - with IP4 the address changes from 0.0.0.0 to 127.0.0.1
     // IPV6 doesn't work at present.
     //
      assert((client.address().address === result.address || 
           client.address().address.concat('1') === result.address), "address should match")
    */
  })

  it('should get our external address from a public server if there is no other server given', async function () {
    const result = await getExternalIp(undefined, servers[0].socket)

    assert(result != undefined, 'server should be able to detect its external address')
  })

  it('should return a valid external address even if some external STUN servers produce a timeout', async function () {
    const before = Date.now()
    const result = await getExternalIp(
      [
        ...PUBLIC_STUN_SERVERS.slice(0, Math.max(0, DEFAULT_PARALLEL_STUN_CALLS - 1)),
        new Multiaddr(`/ip4/127.0.0.1/udp/1`)
      ],
      servers[0].socket
    )

    assert(Date.now() - before >= STUN_TIMEOUT, `should not resolve before timeout ends`)
    assert(result != undefined, `Timeout should not lead to empty result`)
  })

  it('should try other STUN servers after DNS failure', async function () {
    const before = Date.now()
    const response = await getExternalIp(
      [new Multiaddr(`/dns4/totallyinvalidurl.hoprnet.org/udp/12345`)],
      servers[0].socket
    )

    assert(response != undefined, `STUN request must be successful`)

    assert(Date.now() - before >= STUN_TIMEOUT, `STUN request must produce at least one timeout`)
  })

  it('should not try other STUN servers if running locally', async function () {
    const before = Date.now()
    const response = await getExternalIp(
      [new Multiaddr(`/dns4/totallyinvalidurl.hoprnet.org/udp/12345`)],
      servers[0].socket,
      true
    )

    assert(response == undefined, `STUN request must not be successful`)

    assert(Date.now() - before >= STUN_TIMEOUT, `STUN request must produce at least one timeout`)
  })

  it('should understand ambiguous results', async function () {
    const BASE_PUBLIC_ADDRESS = `1.2.3.`
    const tweakedServers = await Promise.all(
      Array.from({ length: 2 }, (_, index: number) =>
        getAmbiguousSTUNServer(index, BASE_PUBLIC_ADDRESS.concat(index.toString()))
      )
    )

    const response = await getExternalIp(
      tweakedServers.map((socket: Socket) => new Multiaddr(`/ip4/127.0.0.1/udp/${socket.address().port}`)),
      servers[0].socket
    )

    assert(response == undefined, `Ambiguous results from local STUN servers must detected as bidirectional NAT`)

    await Promise.all(tweakedServers.map(closeSTUNServer))
  })

  it('should understand ambiguous results when running in local testnet', async function () {
    const tweakedServers = await Promise.all(
      Array.from({ length: 2 }, (_, index: number) => getAmbiguousSTUNServer(index))
    )

    const responseWhenRunningLocally = await getExternalIp(
      tweakedServers.map((socket: Socket) => new Multiaddr(`/ip4/127.0.0.1/udp/${socket.address().port}`)),
      servers[0].socket,
      true
    )

    assert(
      responseWhenRunningLocally == undefined,
      `Ambiguous results from local STUN servers should not lead to successful STUN response`
    )

    await Promise.all(tweakedServers.map(closeSTUNServer))
  })

  it('should return local IP address when running in local testnet', async function () {
    const responseWhenRunningLocally = await getExternalIp(
      servers.slice(1).map((server: ServerType) => new Multiaddr(`/ip4/127.0.0.1/udp/${server.socket.address().port}`)),
      servers[0].socket,
      true
    )

    assert(responseWhenRunningLocally != undefined, 'STUN request must be successful')

    assert(
      responseWhenRunningLocally.address === '127.0.0.1',
      `When running all nodes on localhost, response must be localhost`
    )
  })

  it('should not fail on DNS failures', async function () {
    const stunResult = await getExternalIp(
      [
        new Multiaddr(`/dns4/totallyinvalidurl.hoprnet.org/udp/12345`),
        ...PUBLIC_STUN_SERVERS.slice(DEFAULT_PARALLEL_STUN_CALLS - 1)
      ],
      servers[0].socket
    )

    assert(stunResult != undefined, `STUN request should work even if there are DNS failures`)
  })

  it('should contact only a few STUN servers', async function () {
    const multiaddrs = servers
      .slice(1)
      .map((server: ServerType) => Multiaddr.fromNodeAddress(nodeToMultiaddr(server.socket.address()), 'udp'))

    assert(multiaddrs.length == DEFAULT_PARALLEL_STUN_CALLS + 1)

    const stunResult = await getExternalIp(multiaddrs, servers[0].socket)

    assert(stunResult != undefined, `STUN requests must lead to a result`)

    let contactedPromises = servers.slice(1).map((server) => server.gotContacted.promise)
    const contactedIndices: number[] = []

    for (let i = 0; i < DEFAULT_PARALLEL_STUN_CALLS; i++) {
      const next = await Promise.race(contactedPromises)

      contactedIndices.push(next)
      contactedPromises = servers
        .slice(1)
        .filter((server: ServerType) => !contactedIndices.includes(server.index))
        .map((server: ServerType) => server.gotContacted.promise)
    }

    assert(
      servers.some((server: ServerType) => !contactedIndices.includes(server.index) && server.contactCount == 0),
      `At least one server should not have been contacted`
    )
  })

  after(async () => {
    await Promise.all(
      servers.map((server) => {
        // Make sure that there are no hanging promises
        server.gotContacted.resolve()

        server.socket.close()
        return once(server.socket, 'close')
      })
    )
  })
})
