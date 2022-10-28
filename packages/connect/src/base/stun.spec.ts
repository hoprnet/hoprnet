import type { RemoteInfo } from 'dgram'
import {
  handleStunRequest,
  DEFAULT_PARALLEL_STUN_CALLS,
  PUBLIC_STUN_SERVERS,
  STUN_TIMEOUT,
  iterateThroughStunServers,
  performSTUNRequests,
  getUsableResults,
  type Request,
  intepreteResults,
  getExternalIp
} from './stun.js'
import { Multiaddr } from '@multiformats/multiaddr'
import assert from 'assert'
import { defer, type DeferType } from '@hoprnet/hopr-utils'
import { stopNode, startStunServer, bindToUdpSocket } from './utils.spec.js'

/**
 * Creates a STUN server that answers with tweaked STUN responses to simulate
 * ambiguous results from STUN servers
 * @param port fake port
 * @param address fake address
 * @returns STUN server answering with falsy responses
 */
async function getAmbiguousSTUNServer(
  port: number | undefined,
  address: string | undefined = undefined,
  state: { msgReceived: DeferType<void>; contactCount?: number } | undefined,
  reply: boolean = true
) {
  const socket = await bindToUdpSocket(undefined)

  socket.on('message', (msg: Buffer, rinfo: RemoteInfo) => {
    if (reply) {
      handleStunRequest(socket, msg, rinfo, {
        ...rinfo,
        address: address ?? rinfo.address,
        port: port ?? rinfo.port
      })
    }
    if (state?.contactCount != undefined) {
      state.contactCount += 1
    }
    state?.msgReceived.resolve()
  })

  return socket
}

type StateType = {
  msgReceived: DeferType<void>
  contactCount: number
}
function getState(amount: number): StateType[] {
  return Array.from({ length: amount }, (_) => ({ msgReceived: defer<void>(), contactCount: 0 }))
}

describe('test STUN helper functions', function () {
  it('iteratively contact STUN servers', async function () {
    this.timeout(3 * STUN_TIMEOUT + 2e3)
    const AMOUNT = 2 * DEFAULT_PARALLEL_STUN_CALLS + 1
    const states = getState(AMOUNT)
    const servers = await Promise.all(
      Array.from({ length: AMOUNT }, (_, index: number) =>
        getAmbiguousSTUNServer(index, undefined, states[index], index < 1)
      )
    )

    const socket = await bindToUdpSocket()

    const responses = await iterateThroughStunServers(
      servers.map((s) => new Multiaddr(`/ip4/127.0.0.1/udp/${s.address().port}`)),
      socket,
      Infinity,
      true
    )

    await Promise.all(states.map((state: StateType) => state.msgReceived.promise))

    assert(
      responses != undefined && responses.length == 1,
      `Must contain only one response because the other STUN servers were not responding`
    )
    await Promise.all(servers.concat(socket).map(stopNode))
  })

  it('iteratively contact STUN servers and get ambiguous results', async function () {
    this.timeout(3 * STUN_TIMEOUT + 2e3)
    const AMOUNT = 2
    const states = getState(AMOUNT)
    const servers = await Promise.all(
      Array.from({ length: AMOUNT }, (_, index: number) =>
        getAmbiguousSTUNServer(index, undefined, states[index], true)
      )
    )

    const socket = await bindToUdpSocket()

    const responses = await iterateThroughStunServers(
      servers.map((s) => new Multiaddr(`/ip4/127.0.0.1/udp/${s.address().port}`)),
      socket,
      Infinity,
      true
    )

    assert(responses != undefined && responses.length == 2, `Must return two responses`)
    assert(
      responses[0].response != undefined && responses[1].response != undefined,
      `Both request must lead to a response`
    )
    assert(responses[0].response.port != responses[1].response.port, `Responses must be different`)

    await Promise.all(states.map((state: StateType) => state.msgReceived.promise))

    await Promise.all(servers.concat(socket).map(stopNode))
  })

  it('iteratively contact STUN servers with limit', async function () {
    this.timeout(3 * STUN_TIMEOUT + 2e3)
    const AMOUNT = DEFAULT_PARALLEL_STUN_CALLS * 2 + 1
    const states = getState(AMOUNT)
    const servers = await Promise.all(
      Array.from({ length: AMOUNT }, (_, index: number) =>
        getAmbiguousSTUNServer(undefined, undefined, states[index], index < 1)
      )
    )

    const socket = await bindToUdpSocket()

    const LIMIT = 5

    await iterateThroughStunServers(
      servers.map((s) => new Multiaddr(`/ip4/127.0.0.1/udp/${s.address().port}`)),
      socket,
      LIMIT
    )

    const indices = states.reduce((acc: number[], state: StateType, index: number) => {
      if (state.contactCount == 0) {
        acc.push(index)
      }
      return acc
    }, [])

    assert(indices.length >= AMOUNT - LIMIT)

    await Promise.all(
      states.map((state: StateType, i: number) => {
        if (indices.includes(i)) {
          return Promise.resolve()
        } else {
          return state.msgReceived.promise
        }
      })
    )

    await Promise.all(servers.concat(socket).map(stopNode))
  })

  it('dns failures', async function () {
    const socket = await bindToUdpSocket()

    let responses: Request[] | undefined
    await assert.doesNotReject(async () => {
      responses = await performSTUNRequests([new Multiaddr(`/dns4/totallyinvalidurl.hoprnet.org/udp/12345`)], socket)
    }, `dns error must not cause an exception`)

    assert(responses != undefined && responses.length == 0, `Failed STUN request must not return any response`)
    await stopNode(socket)
  })

  it('result filter', function () {
    const ip6address = {
      family: 'IPv6',
      address: '::1',
      port: 0
    }

    assert(
      getUsableResults([{ response: ip6address }] as Request[], true).length == 0,
      `Must not accept IPv6 addresses in local-mode`
    )
    assert(
      getUsableResults([{ response: ip6address }] as Request[], false).length == 0,
      `Must not accept IPv6 addresses in normal-mode`
    )

    const localhostAddress = {
      family: 'IPv4',
      address: '127.0.0.1',
      port: 0
    }

    assert(
      getUsableResults([{ response: localhostAddress }] as Request[], true).length == 1,
      `Must accept localhost addresses in local-mode`
    )
    assert(
      getUsableResults([{ response: localhostAddress }] as Request[], false).length == 0,
      `Must not accept localhost addresses in normal-mode`
    )

    const localAddress = {
      family: 'IPv4',
      address: '192.168.0.23',
      port: 0
    }

    assert(
      getUsableResults([{ response: localAddress }] as Request[], true).length == 1,
      `Must accept local addresses in local-mode`
    )
    assert(
      getUsableResults([{ response: localAddress }] as Request[], false).length == 0,
      `Must not accept local addresses in normal-mode`
    )

    const publicAddress = {
      family: 'IPv4',
      address: '1.2.3.4',
      port: 0
    }

    assert(
      getUsableResults([{ response: publicAddress }] as Request[], true).length == 0,
      `Must not accept public addresses in local-mode`
    )
    assert(
      getUsableResults([{ response: publicAddress }] as Request[], false).length == 1,
      `Must accept public addresses in normal-mode`
    )

    assert(getUsableResults([] as Request[]).length == 0, `Must not accept empty responses`)
  })

  it('check ambiguity detection', function () {
    const ambiguousResults: Pick<Request, 'response'>[] = [
      {
        response: {
          family: 'IPv4',
          address: '1.2.3.4',
          port: 0
        }
      },
      {
        response: {
          family: 'IPv4',
          address: '1.2.3.4',
          port: 1
        }
      }
    ]

    assert(intepreteResults(ambiguousResults as Required<Request>[]).ambiguous == true)

    const nonAmbiguousResults: Pick<Request, 'response'>[] = [
      {
        response: {
          family: 'IPv4',
          address: '1.2.3.4',
          port: 0
        }
      },
      {
        response: {
          family: 'IPv4',
          address: '1.2.3.4',
          port: 0
        }
      }
    ]

    assert(intepreteResults(nonAmbiguousResults as Required<Request>[]).ambiguous == false)
  })
})

describe('test getExternalIp', function () {
  it('return an address in local-mode if no STUN servers are given', async function () {
    const socket = await bindToUdpSocket()

    const result = await getExternalIp(undefined, socket, true)

    assert(result != undefined, `local-mode should lead to a valid external IP`)

    await stopNode(socket)
  })

  it(`return an address in local-mode`, async function () {
    const AMOUNT = 3
    const servers = await Promise.all(Array.from({ length: AMOUNT }, () => startStunServer(undefined, undefined)))

    const socket = await bindToUdpSocket()

    const result = await getExternalIp(
      servers.map((s) => new Multiaddr(`/ip4/127.0.0.1/udp/${s.address().port}`)),
      socket,
      true
    )

    assert(result != undefined, `local-mode should lead to a valid external IP`)

    await Promise.all(servers.concat(socket).map(stopNode))
  })

  it(`return no address in local-mode if results are ambiguous`, async function () {
    const AMOUNT = 3
    const servers = await Promise.all(
      Array.from({ length: AMOUNT }, (_, index: number) => getAmbiguousSTUNServer(index, undefined, undefined))
    )

    const socket = await bindToUdpSocket()

    const result = await getExternalIp(
      servers.map((s) => new Multiaddr(`/ip4/127.0.0.1/udp/${s.address().port}`)),
      socket,
      true
    )

    assert(result == undefined, `ambiguos results in local-mode should not lead to an address`)

    await Promise.all(servers.concat(socket).map(stopNode))
  })

  it(`return no address in local-mode if only one STUN server answers`, async function () {
    const AMOUNT = 3
    const servers = await Promise.all(
      Array.from({ length: AMOUNT }, (_, index: number) =>
        getAmbiguousSTUNServer(index, undefined, undefined, index < 1)
      )
    )

    const socket = await bindToUdpSocket()

    const result = await getExternalIp(
      servers.map((s) => new Multiaddr(`/ip4/127.0.0.1/udp/${s.address().port}`)),
      socket,
      true
    )

    assert(result == undefined, `ambiguos results in local-mode should not lead to an address`)

    await Promise.all(servers.concat(socket).map(stopNode))
  })

  it(`get the external IP`, async function () {
    this.timeout((DEFAULT_PARALLEL_STUN_CALLS / DEFAULT_PARALLEL_STUN_CALLS) * STUN_TIMEOUT + 2e3)
    const socket = await bindToUdpSocket()

    const results = await iterateThroughStunServers(PUBLIC_STUN_SERVERS, socket)

    if (results.length == 0) {
      console.log(`Node cannot reach more than one external STUN servers. Has the node access to the internet?`)
      // Cannot proceed without access to internet
      return
    }

    const interpreted = intepreteResults(results)

    if (interpreted.ambiguous) {
      console.log(`Node seems to run behind a bidirectional NAT. External IP address is ambigous`)
      return
    }

    const externalIP = await getExternalIp(undefined, socket)

    assert(externalIP != undefined, `Must return an external address if not running behind a bidirectional NAT`)

    await stopNode(socket)
  })
})
