import { Acknowledgement } from './acknowledgement'
import { AcknowledgementChallenge } from './acknowledgementChallenge'
import { SECRET_LENGTH } from './constants'
import { randomBytes } from 'crypto'
import { deriveAckKeyShare, HalfKey } from '@hoprnet/hopr-utils'
import assert from 'assert'

import PeerId from 'peer-id'

describe('acknowledement message', function () {
  it('create, serialize and deserialize', async function () {
    const AMOUNT = 2
    const [self, counterparty] = await Promise.all(
      Array.from({ length: AMOUNT }).map((_) => PeerId.create({ keyType: 'secp256k1' }))
    )

    const ackKey = new HalfKey(Uint8Array.from(randomBytes(SECRET_LENGTH)))

    const challenge = AcknowledgementChallenge.create(ackKey.toChallenge(), self)

    assert(
      Acknowledgement.deserialize(
        Acknowledgement.create(challenge, ackKey, counterparty).serialize(),
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
        Acknowledgement.create(
          randomBytes(AcknowledgementChallenge.SIZE) as any,
          deriveAckKeyShare(key),
          counterparty
        ).serialize(),
        self,
        counterparty
      )
    )
  })
})
