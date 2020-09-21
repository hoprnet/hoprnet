import assert from 'assert'

import PeerInfo from 'peer-info'
import PeerId from 'peer-id'

const Multiaddr = require('multiaddr')

import { serializePeerInfo, deserializePeerInfo } from '.'

describe('test peerInfo serialisation', function () {
  it('should serialize and deserilize a peerInfo', async function () {
    const peerInfo = await PeerInfo.create(await PeerId.create({ keyType: 'secp256k1' }))

    assert(
      (await deserializePeerInfo(Buffer.from(serializePeerInfo(peerInfo)))).id.toB58String() ==
        peerInfo.id.toB58String(),
      `Serialized peerInfo should be deserializable and id should match.`
    )

    const testMultiaddr = Multiaddr('/ip4/127.0.0.1/tcp/0')
    peerInfo.multiaddrs.add(testMultiaddr)

    assert(
      (await deserializePeerInfo(Buffer.from(serializePeerInfo(peerInfo)))).multiaddrs.has(testMultiaddr),
      `Serialized peerInfo should be deserializable and multiaddrs should match.`
    )

    const secondTestMultiaddr = Multiaddr('/ip4/127.0.0.1/tcp/1')
    peerInfo.multiaddrs.add(secondTestMultiaddr)

    const thirdTestMultiaddr = Multiaddr('/ip4/127.0.0.1/tcp/2')

    const deserialized = await deserializePeerInfo(Buffer.from(serializePeerInfo(peerInfo)))
    assert(
      deserialized.multiaddrs.has(testMultiaddr) &&
        deserialized.multiaddrs.has(secondTestMultiaddr) &&
        !deserialized.multiaddrs.has(thirdTestMultiaddr),
      `Serialized peerInfo should be deserializable and multiaddrs should match.`
    )
  })
})
