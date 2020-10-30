import {PRP} from './prp'
import {u8aEquals} from '../u8a'
import assert from 'assert'
import {randomBytes} from 'crypto'

describe(`test Pseudo-Random Permutation`, function () {
  it(`should 'encrypt' and 'decrypt' a U8a`, function () {
    const prp = PRP.createPRP(randomBytes(PRP.KEY_LENGTH), randomBytes(PRP.IV_LENGTH))

    const test = randomBytes(200) // turn .slice() into copy
    const ciphertext = prp.permutate(Uint8Array.from(test))

    assert(
      ciphertext.some((value: number, index: number) => value != test[index]),
      'ciphertext should be different from plaintext'
    )

    const plaintext = prp.inverse(ciphertext)
    assert(u8aEquals(plaintext, test), `'encryption' and 'decryption' should yield the plaintext`)
  })

  it(`should 'decrypt' and 'encrypt' a U8a`, function () {
    const prp = PRP.createPRP(randomBytes(PRP.KEY_LENGTH), randomBytes(PRP.IV_LENGTH))

    const test = randomBytes(200) // turn .slice() into copy
    const ciphertext = prp.inverse(Uint8Array.from(test))

    assert(
      ciphertext.some((value: number, index: number) => value != test[index]),
      'ciphertext should be different from plaintext'
    )

    const plaintext = prp.permutate(ciphertext)
    assert(
      plaintext.every((value: number, index: number) => value == test[index]),
      `'decryption' and 'encryption' should yield the plaintext`
    )
  })
})
