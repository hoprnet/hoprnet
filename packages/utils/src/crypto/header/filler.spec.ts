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

    const secrets = Array.from({ length: hops }, (_) => randomBytes(SECRET_LENGTH))

    let header = new Uint8Array(perHop * (hops - 1) + lastHop)

    generateFiller(header, perHop, lastHop, secrets)

    for (let i = 0; i < hops - 1; i++) {
      const blinding = PRG.createPRG(derivePRGParameters(secrets[secrets.length - 2 - i])).digest(
        0,
        lastHop + (secrets.length - 1) * perHop
      )

      u8aXOR(true, header, blinding)

      assert(
        u8aEquals(header.subarray(lastHop + (hops - 2) * perHop), new Uint8Array(perHop)),
        `XORing blinding must erase last bits`
      )

      // Roll header
      header = Uint8Array.from([
        ...new Uint8Array(perHop),
        ...header.slice(0, lastHop + Math.max(0, secrets.length - 2) * perHop)
      ])
    }
  })

  it('generate a filler and verify - edge cases', function () {
    const perHop = 23
    const lastHop = 31
    const hops = 1

    const secrets = Array.from({ length: hops }, (_) => randomBytes(SECRET_LENGTH))

    let header = new Uint8Array(perHop * (hops - 1) + lastHop)

    generateFiller(header, perHop, lastHop, secrets)
    console.log(header)

    assert(
      header.every((x) => x == 0),
      'must not produce fillers for zero-hop packets'
    )

    // Should not throw an error
    generateFiller(new Uint8Array(), perHop, lastHop, [])
  })
})
