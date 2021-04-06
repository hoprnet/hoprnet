import { generateFiller } from './filler'
import { randomBytes } from 'crypto'
import { SECRET_LENGTH } from './constants'
import { u8aXOR, u8aEquals } from '../../u8a'
import { PRG } from '../prg'
import { derivePRGParameters } from './blinding'
import assert from 'assert'

describe('test filler', function () {
  it('generate a filler and verify', function () {
    const perHop = 23
    const lastHop = 31
    const hops = 5
    const maxHops = hops

    const secrets = Array.from({ length: hops }, (_) => randomBytes(SECRET_LENGTH))
    const extendedHeaderLength = perHop * maxHops + lastHop

    let extendedHeader = new Uint8Array(perHop * maxHops + lastHop)

    generateFiller(extendedHeader, maxHops, perHop, lastHop, secrets)

    for (let i = 0; i < hops - 1; i++) {
      const blinding = PRG.createPRG(derivePRGParameters(secrets[secrets.length - 2 - i])).digest(
        0,
        extendedHeaderLength
      )

      u8aXOR(true, extendedHeader, blinding)

      assert(
        u8aEquals(extendedHeader.subarray(extendedHeaderLength - perHop), new Uint8Array(perHop)),
        `XORing blinding must erase last bits`
      )

      // Roll header
      extendedHeader = Uint8Array.from([
        ...new Uint8Array(perHop),
        ...extendedHeader.slice(0, extendedHeaderLength - perHop)
      ])
    }
  })

  it('generate a filler and verify - reduced path', function () {
    const perHop = 23
    const lastHop = 31
    const hops = 4
    const maxHops = 5

    const secrets = Array.from({ length: hops }, (_) => randomBytes(SECRET_LENGTH))
    const extendedHeaderLength = perHop * maxHops + lastHop

    let header = new Uint8Array(extendedHeaderLength)

    generateFiller(header, maxHops, perHop, lastHop, secrets)

    assert(header.slice(0, lastHop + (maxHops - hops) * perHop).every((x) => x == 0))

    for (let i = 0; i < hops - 1; i++) {
      const blinding = PRG.createPRG(derivePRGParameters(secrets[secrets.length - 2 - i])).digest(
        0,
        extendedHeaderLength
      )

      u8aXOR(true, header, blinding)

      assert(
        u8aEquals(header.subarray(extendedHeaderLength - perHop), new Uint8Array(perHop)),
        `XORing blinding must erase last bits`
      )

      // Roll header
      header = Uint8Array.from([...new Uint8Array(perHop), ...header.slice(0, extendedHeaderLength - perHop)])
    }
  })

  it('generate a filler and verify - edge cases', function () {
    const perHop = 23
    const lastHop = 31
    const hops = 1
    const maxHops = hops

    const secrets = Array.from({ length: hops }, (_) => randomBytes(SECRET_LENGTH))

    let extendedHeader = new Uint8Array(perHop * hops + lastHop)

    generateFiller(extendedHeader, maxHops, perHop, lastHop, secrets)

    assert(
      extendedHeader.every((x) => x == 0),
      'must not produce fillers for zero-hop packets'
    )

    // Should not throw an error
    generateFiller(new Uint8Array(), 0, perHop, lastHop, [])
  })
})
