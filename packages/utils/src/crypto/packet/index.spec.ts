import PeerId from 'peer-id'
import { randomBytes } from 'crypto'
import { getPacketLength, createPacket, forwardTransform, generateKeyShares } from '.'
import { PAYLOAD_SIZE } from './constants'
import assert from 'assert'
import { u8aEquals } from '../../u8a'
import { PADDING_TAG_LENGTH } from './padding'

describe('header', function () {
  it('create a header and transform', async function () {
    const AMOUNT = 13
    const maxHops = 13

    const path = await Promise.all(Array.from({ length: AMOUNT }, (_) => PeerId.create({ keyType: 'secp256k1' })))

    const testMsg = Uint8Array.from(randomBytes(PAYLOAD_SIZE - PADDING_TAG_LENGTH))

    const { alpha, secrets } = generateKeyShares(path)

    let packet = createPacket(
      secrets,
      alpha,
      Uint8Array.from(testMsg), // clone testMsg
      path,
      maxHops,
      0,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    assert(packet.length == getPacketLength(maxHops, 0, 0))

    for (const [index, peer] of path.entries()) {
      const result = forwardTransform(peer, packet, 0, 0, maxHops)

      assert(u8aEquals(result.derivedSecret, secrets[index]), `Derived secret must be identical to created secret`)

      if (index == path.length - 1) {
        assert(result.lastNode == true, `Implementation must detect final recipient`)

        assert(u8aEquals(result.plaintext, testMsg), `decoded message must be identical to input message`)
      } else {
        assert(result.lastNode == false, `Implementation must detect message as forward message`)

        assert(u8aEquals(result.nextHop, path[index + 1].pubKey.marshal()), `Next hop must be next pubKey`)

        assert(result.additionalRelayData.length == 0)

        packet = result.packet
      }
    }
  })

  it('create a header and transform - reduced path', async function () {
    const AMOUNT = 11
    const maxHops = 13

    const path = await Promise.all(Array.from({ length: AMOUNT }, (_) => PeerId.create({ keyType: 'secp256k1' })))

    const testMsg = Uint8Array.from(randomBytes(PAYLOAD_SIZE - PADDING_TAG_LENGTH))

    const { alpha, secrets } = generateKeyShares(path)

    let packet = createPacket(
      secrets,
      alpha,
      Uint8Array.from(testMsg), // clone testMsg
      path,
      maxHops,
      0,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    for (const [index, peer] of path.entries()) {
      const result = forwardTransform(peer, packet, 0, 0, maxHops)

      if (index == path.length - 1) {
        assert(result.lastNode == true, `Implementation must detect final recipient`)

        assert(u8aEquals(result.plaintext, testMsg), `decoded message must be identical to input message`)
      } else {
        assert(result.lastNode == false, `Implementation must detect message as forward message`)

        assert(u8aEquals(result.nextHop, path[index + 1].pubKey.marshal()), `Next hop must be next pubKey`)

        assert(result.additionalRelayData.length == 0)

        packet = result.packet
      }
    }
  })
})
