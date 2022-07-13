import { generateKeyShares, forwardTransform } from './keyShares.js'
import { u8aEquals } from '../../u8a/index.js'

import { createSecp256k1PeerId } from '@libp2p/peer-id-factory'
import assert from 'assert'

describe('test key share generation', function () {
  it('generate key shares and verify them', async function () {
    const AMOUNT = 4
    const keyPairs = await Promise.all(Array.from({ length: AMOUNT }).map((_) => createSecp256k1PeerId()))

    const { alpha, secrets } = generateKeyShares(keyPairs)

    for (let i = 0; i < AMOUNT; i++) {
      const { alpha: tmpAlpha, secret } = forwardTransform(alpha, keyPairs[i])

      assert(u8aEquals(secret, secrets[i]))

      alpha.set(tmpAlpha)
    }
  })

  it('generate key shares and verify them - false-posivitive test', async function () {
    const AMOUNT = 3
    const keyPairs = await Promise.all(Array.from({ length: AMOUNT }).map((_) => createSecp256k1PeerId()))

    const { alpha, secrets } = generateKeyShares(keyPairs)

    assert(!u8aEquals(secrets[0], secrets[1], ...secrets.slice(2)), 'Secrets must be different')

    for (let i = 0; i < AMOUNT; i++) {
      const { alpha: tmpAlpha, secret } = forwardTransform(alpha, keyPairs[i])

      assert(u8aEquals(secret, secrets[i]))

      assert(!u8aEquals(alpha, tmpAlpha), 'alpha must change')

      alpha.set(tmpAlpha)
    }
  })
})
