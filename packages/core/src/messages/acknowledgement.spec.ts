import { Acknowledgement } from './acknowledgement'
import { Challenge } from './challenge'
import { randomBytes } from 'crypto'
import { SECRET_LENGTH } from './constants'
import { deriveAckKeyShare } from '@hoprnet/hopr-utils'
import { publicKeyCreate } from 'secp256k1'
import assert from 'assert'

import PeerId from 'peer-id'

describe('acknowledement message', function () {
  it('create, serialize and deserialize', async function () {
    const AMOUNT = 2
    const [self, counterparty] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const key = randomBytes(SECRET_LENGTH)
    const ackKey = deriveAckKeyShare(key)

    const challenge = Challenge.create(publicKeyCreate(ackKey), self)

    assert(
      Acknowledgement.deserialize(
        Acknowledgement.create(challenge, key, counterparty).serialize(),
        self,
        counterparty
      ) != null
    )
  })

  it('create, serialize and deserialize - false positives', async function () {
    const AMOUNT = 2
    const [self, counterparty] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const key = randomBytes(SECRET_LENGTH)

    assert.throws(() =>
      Acknowledgement.deserialize(
        Acknowledgement.create(randomBytes(Challenge.SIZE) as any, key, counterparty).serialize(),
        self,
        counterparty
      )
    )
  })
})
