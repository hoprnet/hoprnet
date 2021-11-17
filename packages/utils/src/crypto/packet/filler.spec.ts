import { generateFiller } from './filler'
import { randomBytes } from 'crypto'
import { PRESECRET_LENGTH } from './constants'
import { u8aXOR } from '../../u8a'
import { PRG } from '../prg'
import { derivePRGParameters } from './keyDerivation'
import assert from 'assert'

describe('test filler', function () {
  it('generate a filler and verify', function () {
    const perHop = 3
    const lastHop = 5
    const hops = 3
    const maxHops = hops

    const secrets = Array.from({ length: hops }, (_) => randomBytes(PRESECRET_LENGTH))
    const extendedHeaderLength = perHop * maxHops + lastHop
    const headerLength = perHop * (maxHops - 1) + lastHop

    let extendedHeader = new Uint8Array(perHop * maxHops + lastHop)

    extendedHeader.set(generateFiller(maxHops, perHop, lastHop, secrets), lastHop)

    assert(extendedHeader.subarray(0, lastHop).every((x) => x == 0))

    extendedHeader.copyWithin(perHop, 0, headerLength)

    for (let i = 0; i < hops - 1; i++) {
      const blinding = PRG.createPRG(derivePRGParameters(secrets[secrets.length - 2 - i])).digest(
        0,
        extendedHeaderLength
      )

      u8aXOR(true, extendedHeader, blinding)

      assert(
        extendedHeader.subarray(headerLength).every((x) => x == 0),
        `XORing blinding must erase last bits`
      )

      // Roll header
      extendedHeader.copyWithin(perHop, 0, headerLength)
    }
  })

  it('generate a filler and verify - reduced path', function () {
    const perHop = 3
    const lastHop = 5
    const hops = 4
    const maxHops = 5

    const secrets = Array.from({ length: hops }, (_) => randomBytes(PRESECRET_LENGTH))
    const extendedHeaderLength = perHop * maxHops + lastHop
    const headerLength = perHop * (maxHops - 1) + lastHop
    const paddingLength = perHop * (maxHops - hops)

    let extendedHeader = new Uint8Array(extendedHeaderLength)

    extendedHeader.set(generateFiller(maxHops, perHop, lastHop, secrets), lastHop + paddingLength)

    assert(extendedHeader.slice(0, lastHop + (maxHops - hops) * perHop).every((x) => x == 0))

    extendedHeader.copyWithin(perHop, 0, headerLength)

    for (let i = 0; i < hops - 1; i++) {
      const blinding = PRG.createPRG(derivePRGParameters(secrets[secrets.length - 2 - i])).digest(
        0,
        extendedHeaderLength
      )

      u8aXOR(true, extendedHeader, blinding)

      assert(
        extendedHeader.subarray(headerLength).every((x) => x == 0),
        `XORing blinding must erase last bits`
      )

      // Roll header
      extendedHeader.copyWithin(perHop, 0, headerLength)
    }
  })

  it('generate a filler and verify - edge cases', function () {
    const perHop = 23
    const lastHop = 31
    const hops = 1
    const maxHops = hops

    const secrets = Array.from({ length: hops }, (_) => randomBytes(PRESECRET_LENGTH))

    const firstFiller = generateFiller(maxHops, perHop, lastHop, secrets)

    assert(firstFiller == undefined || firstFiller.length == 0, 'must not produce fillers for zero-hop packets')

    // Should not throw an error
    const secondFiller = generateFiller(0, perHop, lastHop, [])

    assert(secondFiller == undefined || secondFiller.length == 0, `must not mutate any memory on zero-length paths`)
  })
})
