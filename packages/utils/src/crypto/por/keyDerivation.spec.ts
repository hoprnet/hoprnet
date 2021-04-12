import { SECRET_LENGTH } from './constants'
import { sampleFieldElement } from './keyDerivation'
import assert from 'assert'

describe(`PoR key derivation`, function () {
  it('derive proper field elements', function () {
    const WORKING_SECRET = new Uint8Array(SECRET_LENGTH).fill(255)
    const HASH_KEY = 'my hash key'

    const INVALID_ELEMENT = new Uint8Array(SECRET_LENGTH).fill(255)
    const VALID_ELEMENT = new Uint8Array(SECRET_LENGTH).fill(254)

    sampleFieldElement(WORKING_SECRET, HASH_KEY)

    let hashKeyChanged = false

    // Check that sampleFieldElement is indeed changing the hashkey
    sampleFieldElement(WORKING_SECRET, HASH_KEY, (hashKey: string) => {
      if (hashKey === HASH_KEY) {
        return INVALID_ELEMENT
      } else {
        hashKeyChanged = true
        return VALID_ELEMENT
      }
    })

    assert(hashKeyChanged)
  })
})
