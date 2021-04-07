import PeerId from 'peer-id'
import { randomBytes } from 'crypto'
import { createPacket, forwardTransform } from '.'
import { PAYLOAD_SIZE } from './constants'
import assert from 'assert'
import { u8aEquals } from '../../u8a'

describe('header', function () {
  it('create a header and transform', async function () {
    const AMOUNT = 13
    const maxHops = 13

    const path = await Promise.all(Array.from({ length: AMOUNT }, (_) => PeerId.create({ keyType: 'secp256k1' })))

    const testMsg = Uint8Array.from(randomBytes(PAYLOAD_SIZE))

    let packet = createPacket(
      Uint8Array.from(testMsg), // clone testMsg
      path,
      maxHops,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    for (const [index, peer] of path.entries()) {
      const result = forwardTransform(peer, packet, 0, 0, maxHops)

      if (index == path.length - 1) {
        assert(result[0] == true, `Implementation must detect final recipient`)

        assert(u8aEquals(result[1], testMsg), `decoded message must be identical to input message`)
      } else {
        assert(result[0] == false, `Implementation must detect message as forward message`)

        assert(u8aEquals(result[2], path[index + 1].pubKey.marshal()), `Next hop must be next pubKey`)

        assert(result[3].length == 0)

        packet = result[1]
      }
    }
  })

  it('create a header and transform - reduced path', async function () {
    const AMOUNT = 11
    const maxHops = 13

    const path = await Promise.all(Array.from({ length: AMOUNT }, (_) => PeerId.create({ keyType: 'secp256k1' })))

    const testMsg = Uint8Array.from(randomBytes(PAYLOAD_SIZE))

    let packet = createPacket(
      Uint8Array.from(testMsg), // clone testMsg
      path,
      maxHops,
      Array.from({ length: AMOUNT }, (_) => new Uint8Array()),
      new Uint8Array()
    )

    for (const [index, peer] of path.entries()) {
      const result = forwardTransform(peer, packet, 0, 0, maxHops)

      if (index == path.length - 1) {
        assert(result[0] == true, `Implementation must detect final recipient`)

        assert(u8aEquals(result[1], testMsg), `decoded message must be identical to input message`)
      } else {
        assert(result[0] == false, `Implementation must detect message as forward message`)

        assert(u8aEquals(result[2], path[index + 1].pubKey.marshal()), `Next hop must be next pubKey`)

        assert(result[3].length == 0)

        packet = result[1]
      }
    }
  })
})
