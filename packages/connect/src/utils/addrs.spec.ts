import assert from 'assert'
import { parseAddress } from './addrs.js'
import { Multiaddr } from '@multiformats/multiaddr'

const firstPeer = `16Uiu2HAmCPgzWWQWNAn2E3UXx1G3CMzxbPfLr1SFzKqnFjDcbdwg`
const secondPeer = `16Uiu2HAkyvdVZtG8btak5SLrxP31npfJo6maopj8xwx5XQhKfspb`

describe('test address parsing', function () {
  it('good examples', function () {
    assert(parseAddress(new Multiaddr('/ip4/127.0.0.1/tcp/0')).valid === true)

    assert(parseAddress(new Multiaddr(`/ip4/127.0.0.1/tcp/0/p2p/${firstPeer}`)).valid === true)

    assert(parseAddress(new Multiaddr('/ip6/::1/tcp/0')).valid === true)

    assert(parseAddress(new Multiaddr(`/ip6/::1/tcp/0/p2p/${firstPeer}`)).valid === true)

    assert(parseAddress(new Multiaddr(`/p2p/${secondPeer}/p2p-circuit/p2p/${firstPeer}`)).valid === true)

    assert(parseAddress(new Multiaddr(`/p2p/${secondPeer}/p2p-circuit`)).valid === true)
  })

  it('bad examples', function () {
    assert(parseAddress(new Multiaddr('/tcp/0')).valid === false)

    assert(parseAddress(new Multiaddr('/ip4/127.0.0.1/udp/0')).valid === false)

    assert(parseAddress(new Multiaddr('/ip4/127.0.0.1')).valid === false)

    assert(parseAddress(new Multiaddr('/ip6/::1/udp/0')).valid === false)

    assert(parseAddress(new Multiaddr('/ip6/::1')).valid === false)

    assert(parseAddress(new Multiaddr(`/p2p/${secondPeer}`)).valid === false)

    assert(parseAddress(new Multiaddr(`/p2p/${secondPeer}/tcp/0`)).valid === false)
  })
})
