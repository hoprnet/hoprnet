import assert from 'assert'
import type HoprEthereum from '.'
import * as DbKeys from './dbKeys'
import * as Utils from './utils'
import * as Types from './types'
import PreImage, { GIANT_STEP_WIDTH, TOTAL_ITERATIONS } from './preImage'
import { randomInteger, u8aEquals, durations, u8aToHex } from '@hoprnet/hopr-utils'
import Memdown from 'memdown'
import LevelUp from 'levelup'

function generateConnector(): HoprEthereum {
  return ({
    db: LevelUp(Memdown()),
    dbKeys: DbKeys,
    utils: Utils,
  } as unknown) as HoprEthereum
}

describe('test preImage management', function () {
  this.timeout(durations.seconds(7))
  const connector = generateConnector()
  const preImage = new PreImage(connector)

  const checkIndex = async (index: number, masterSecret: Uint8Array, shouldThrow: boolean) => {
    let hash = new Types.Hash(masterSecret)
    for (let i = 0; i < index; i++) {
      hash = await connector.utils.hash(hash)
    }

    let result,
      errThrown = false
    try {
      result = await preImage.getPreimage(hash)
    } catch (err) {
      errThrown = true
    }

    if (shouldThrow) {
      assert(errThrown, `Must throw an error`)
    } else {
      assert(u8aEquals(await connector.utils.hash(result.preImage), hash) && index == result.index + 1)
    }
  }

  it('should generate a hashed secret and recover a pre-Image', async function () {
    await preImage.create()

    for (let i = 0; i < TOTAL_ITERATIONS / GIANT_STEP_WIDTH; i++) {
      assert(
        (await connector.db.get(Buffer.from(connector.dbKeys.OnChainSecretIntermediary(i * GIANT_STEP_WIDTH)))) != null
      )
    }

    const masterSecret = await connector.db.get(Buffer.from(connector.dbKeys.OnChainSecretIntermediary(0)))

    checkIndex(1, masterSecret, false)
    checkIndex(randomInteger(1, TOTAL_ITERATIONS), masterSecret, false)
    checkIndex(TOTAL_ITERATIONS, masterSecret, false)

    checkIndex(0, masterSecret, true)
    checkIndex(TOTAL_ITERATIONS + 1, masterSecret, true)
  })
})
