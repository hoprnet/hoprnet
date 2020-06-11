import HoprEthereum from '.'
import { Hash } from './types'

import { randomBytes } from 'crypto'
import { u8aEquals, u8aToHex } from '@hoprnet/hopr-utils'

export const GIANT_STEP_WIDTH = 10000
export const TOTAL_ITERATIONS = 100000
class HashedSecret {
  constructor(private coreConnector: HoprEthereum) {}

  async create(): Promise<Hash> {
    let onChainSecret = new Hash(randomBytes(Hash.SIZE))
    let onChainSecretIntermediary = onChainSecret
    let dbBatch = this.coreConnector.db.batch()

    for (let i = 0; i < TOTAL_ITERATIONS; i++) {
      if (i % GIANT_STEP_WIDTH == 0) {
        dbBatch = dbBatch.put(
          Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(i)),
          Buffer.from(onChainSecretIntermediary)
        )
      }
      onChainSecretIntermediary = await this.coreConnector.utils.hash(onChainSecretIntermediary)
    }

    await dbBatch.write()

    return onChainSecretIntermediary
  }

  async getPreimage(hash: Hash): Promise<{ preImage: Uint8Array; index: number }> {
    let closestIntermediary = TOTAL_ITERATIONS - GIANT_STEP_WIDTH
    let intermediary: Hash
    let upperBound = TOTAL_ITERATIONS

    let hashedIntermediary: Hash
    let found = false
    let index: number

    do {
      while (true) {
        try {
          intermediary = await this.coreConnector.db.get(
            Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(closestIntermediary))
          )
          break
        } catch (err) {
          if (err.notFound) {
            if (closestIntermediary == 0) {
              throw Error(`Could not find pre-image`)
            } else {
              closestIntermediary -= GIANT_STEP_WIDTH
            }
          } else {
            throw err
          }
        }
      }

      for (let i = 0; i < upperBound - closestIntermediary; i++) {
        hashedIntermediary = await this.coreConnector.utils.hash(intermediary)
        if (u8aEquals(hashedIntermediary, hash)) {
          found = true
          index = closestIntermediary + i
          break
        } else {
          intermediary = hashedIntermediary
        }
      }

      closestIntermediary -= GIANT_STEP_WIDTH
    } while (!found && closestIntermediary >= 0)

    if (!found) {
      throw Error('notFound')
    }

    return { preImage: intermediary, index }
  }
}

export default HashedSecret
