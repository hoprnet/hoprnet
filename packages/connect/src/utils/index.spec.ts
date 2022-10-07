import { Multiaddr } from '@multiformats/multiaddr'
import assert from 'assert'

import { privKeyToPeerId, u8aEquals } from '@hoprnet/hopr-utils'
import { relayFromRelayAddress, nodeToMultiaddr, toU8aStream } from './index.js'
import { Uint8ArrayList } from 'uint8arraylist'

const SELF = privKeyToPeerId('0x7519c19fd624599c0f97c89913b277a7d1b932d77100f4bbb2bc01969249add3')
const RELAY = privKeyToPeerId('0x152c95bd36e6ada51309558756d5f901853d45ab94336a0bd7bae1453e98ffd6')

describe(`test util functions`, function () {
  it('relay extraction from relay address', function () {
    const extracted = relayFromRelayAddress(new Multiaddr(`/p2p/${RELAY.toString()}/p2p-circuit`))

    assert(extracted.equals(RELAY))

    // Incorrect size
    assert.throws(() => relayFromRelayAddress(new Multiaddr(`/p2p/${SELF.toString()}`)))

    // Incorrect protocol
    assert.throws(() => relayFromRelayAddress(new Multiaddr(`/ip4/127.0.0.1`)))

    // Incorrect size and missing protocol
    assert.throws(() => relayFromRelayAddress(new Multiaddr(`/p2p/${SELF.toString()}`)))
  })

  it('Node.js AddressInfo to Multiaddr', function () {
    const tests = [
      [
        new Multiaddr(`/ip4/127.0.0.1/tcp/12345`),
        nodeToMultiaddr({
          address: '127.0.0.1',
          port: 12345,
          family: 'IPv4'
        })
      ],
      // Accept any *any* address
      [
        new Multiaddr(`/ip4/0.0.0.0/tcp/12345`),
        nodeToMultiaddr({
          address: '::',
          port: 12345,
          family: 'IPv4'
        })
      ],
      [
        new Multiaddr(`/ip4/0.0.0.0/tcp/12345`),
        nodeToMultiaddr({
          address: '0.0.0.0',
          port: 12345,
          family: 'IPv4'
        })
      ],
      [
        new Multiaddr(`/ip6/::1/tcp/12345`),
        nodeToMultiaddr({
          address: '::1',
          port: 12345,
          family: 'IPv6'
        })
      ],
      // Accecpt any *any* address
      [
        new Multiaddr(`/ip6/::/tcp/12345`),
        nodeToMultiaddr({
          address: '0.0.0.0',
          port: 12345,
          family: 'IPv6'
        })
      ],
      [
        new Multiaddr(`/ip6/::/tcp/12345`),
        nodeToMultiaddr({
          address: '::',
          port: 12345,
          family: 'IPv6'
        })
      ]
    ]

    for (const test of tests) {
      assert(
        test[0].equals(test[1]),
        `Expected Multiaddr ${test[0].toString()} must be equal to computed ${test[1].toString()}`
      )
    }

    assert.throws(() => nodeToMultiaddr({ address: '0.0.0.0', family: 'wrongFamily', port: 12345 }))
  })

  it('toU8aStream - string', async function () {
    const firstMessage = 'first message'
    const secondMessage = 'second message'

    const stringStream = (async function* () {
      yield firstMessage
      yield secondMessage
    })()

    let i = 0
    for await (const chunk of toU8aStream(stringStream)) {
      if (i++ == 0) {
        assert(new TextDecoder().decode(chunk) === firstMessage)
      } else {
        assert(new TextDecoder().decode(chunk) === secondMessage)
      }
    }

    const stringStreamSync = (function* () {
      yield firstMessage
      yield secondMessage
    })()

    i = 0
    for (const chunk of toU8aStream(stringStreamSync)) {
      if (i++ == 0) {
        assert(new TextDecoder().decode(chunk) === firstMessage)
      } else {
        assert(new TextDecoder().decode(chunk) === secondMessage)
      }
    }
  })

  it('toU8aStream - Buffer', async function () {
    const firstMessage = Buffer.from([0, 1, 2, 3, 4])
    const secondMessage = Buffer.from([1, 2, 3, 4, 5])

    const stringStream = (async function* () {
      yield firstMessage
      yield secondMessage
    })()

    let i = 0
    for await (const chunk of toU8aStream(stringStream)) {
      assert(!Buffer.isBuffer(chunk))

      if (i++ == 0) {
        assert(u8aEquals(chunk, firstMessage))
      } else {
        assert(u8aEquals(chunk, secondMessage))
      }
    }

    const stringStreamSync = (function* () {
      yield firstMessage
      yield secondMessage
    })()

    i = 0
    for (const chunk of toU8aStream(stringStreamSync)) {
      assert(!Buffer.isBuffer(chunk))
      if (i++ == 0) {
        assert(u8aEquals(chunk, firstMessage))
      } else {
        assert(u8aEquals(chunk, secondMessage))
      }
    }
  })

  it('toU8aStream - Uint8ArrayList', async function () {
    const firstMessage = new Uint8ArrayList(new Uint8Array([0, 1, 2]), new Uint8Array([3, 4]))
    const secondMessage = new Uint8ArrayList(new Uint8Array([1, 2, 3]), new Uint8Array([4, 5]))

    const stringStream = (async function* () {
      yield firstMessage
      yield secondMessage
    })()

    let i = 0
    for await (const chunk of toU8aStream(stringStream)) {
      if (i++ == 0) {
        assert(chunk.length == firstMessage.length)
        assert(u8aEquals(chunk.slice(), firstMessage.slice()))
      } else {
        assert(chunk.length == secondMessage.length)
        assert(u8aEquals(chunk.slice(), secondMessage.slice()))
      }
    }

    const stringStreamSync = (function* () {
      yield firstMessage
      yield secondMessage
    })()

    i = 0
    for (const chunk of toU8aStream(stringStreamSync)) {
      if (i++ == 0) {
        assert(chunk.length == firstMessage.length)
        assert(u8aEquals(chunk.slice(), firstMessage.slice()))
      } else {
        assert(chunk.length == secondMessage.length)
        assert(u8aEquals(chunk.slice(), secondMessage.slice()))
      }
    }
  })

  it('toU8aStream - Uint8Array', async function () {
    const firstMessage = Uint8Array.from([0, 1, 2, 3, 4])
    const secondMessage = Uint8Array.from([1, 2, 3, 4, 5])

    const stringStream = (async function* () {
      yield firstMessage
      yield secondMessage
    })()

    let i = 0
    for await (const chunk of toU8aStream(stringStream)) {
      assert(!Buffer.isBuffer(chunk))

      if (i++ == 0) {
        assert(u8aEquals(chunk, firstMessage))
      } else {
        assert(u8aEquals(chunk, secondMessage))
      }
    }

    const stringStreamSync = (function* () {
      yield firstMessage
      yield secondMessage
    })()

    i = 0
    for (const chunk of toU8aStream(stringStreamSync)) {
      assert(!Buffer.isBuffer(chunk))
      if (i++ == 0) {
        assert(u8aEquals(chunk, firstMessage))
      } else {
        assert(u8aEquals(chunk, secondMessage))
      }
    }
  })

  it('toU8aStream - string', async function () {
    const firstMessage = 'first message'
    const secondMessage = 'second message'

    const stringStream = (async function* () {
      yield firstMessage
      yield secondMessage
    })()

    let i = 0
    for await (const chunk of toU8aStream(stringStream)) {
      if (i++ == 0) {
        assert(new TextDecoder().decode(chunk) === firstMessage)
      } else {
        assert(new TextDecoder().decode(chunk) === secondMessage)
      }
    }

    const stringStreamSync = (function* () {
      yield firstMessage
      yield secondMessage
    })()

    i = 0
    for (const chunk of toU8aStream(stringStreamSync)) {
      if (i++ == 0) {
        assert(new TextDecoder().decode(chunk) === firstMessage)
      } else {
        assert(new TextDecoder().decode(chunk) === secondMessage)
      }
    }
  })

  it('toU8aStream - Buffer', async function () {
    const firstMessage = Buffer.from([0, 1, 2, 3, 4])
    const secondMessage = Buffer.from([1, 2, 3, 4, 5])

    const stringStream = (async function* () {
      yield firstMessage
      yield secondMessage
    })()

    let i = 0
    for await (const chunk of toU8aStream(stringStream)) {
      assert(!Buffer.isBuffer(chunk))

      if (i++ == 0) {
        assert(u8aEquals(chunk, firstMessage))
      } else {
        assert(u8aEquals(chunk, secondMessage))
      }
    }

    const stringStreamSync = (function* () {
      yield firstMessage
      yield secondMessage
    })()

    i = 0
    for (const chunk of toU8aStream(stringStreamSync)) {
      assert(!Buffer.isBuffer(chunk))
      if (i++ == 0) {
        assert(u8aEquals(chunk, firstMessage))
      } else {
        assert(u8aEquals(chunk, secondMessage))
      }
    }
  })

  it('toU8aStream - Uint8ArrayList', async function () {
    const firstMessage = new Uint8ArrayList(new Uint8Array([0, 1, 2]), new Uint8Array([3, 4]))
    const secondMessage = new Uint8ArrayList(new Uint8Array([1, 2, 3]), new Uint8Array([4, 5]))

    const stringStream = (async function* () {
      yield firstMessage
      yield secondMessage
    })()

    let i = 0
    for await (const chunk of toU8aStream(stringStream)) {
      if (i++ == 0) {
        assert(chunk.length == firstMessage.length)
        assert(u8aEquals(chunk.slice(), firstMessage.slice()))
      } else {
        assert(chunk.length == secondMessage.length)
        assert(u8aEquals(chunk.slice(), secondMessage.slice()))
      }
    }

    const stringStreamSync = (function* () {
      yield firstMessage
      yield secondMessage
    })()

    i = 0
    for (const chunk of toU8aStream(stringStreamSync)) {
      if (i++ == 0) {
        assert(chunk.length == firstMessage.length)
        assert(u8aEquals(chunk.slice(), firstMessage.slice()))
      } else {
        assert(chunk.length == secondMessage.length)
        assert(u8aEquals(chunk.slice(), secondMessage.slice()))
      }
    }
  })

  it('toU8aStream - Uint8Array', async function () {
    const firstMessage = Uint8Array.from([0, 1, 2, 3, 4])
    const secondMessage = Uint8Array.from([1, 2, 3, 4, 5])

    const stringStream = (async function* () {
      yield firstMessage
      yield secondMessage
    })()

    let i = 0
    for await (const chunk of toU8aStream(stringStream)) {
      assert(!Buffer.isBuffer(chunk))

      if (i++ == 0) {
        assert(u8aEquals(chunk, firstMessage))
      } else {
        assert(u8aEquals(chunk, secondMessage))
      }
    }

    const stringStreamSync = (function* () {
      yield firstMessage
      yield secondMessage
    })()

    i = 0
    for (const chunk of toU8aStream(stringStreamSync)) {
      assert(!Buffer.isBuffer(chunk))
      if (i++ == 0) {
        assert(u8aEquals(chunk, firstMessage))
      } else {
        assert(u8aEquals(chunk, secondMessage))
      }
    }
  })
})
