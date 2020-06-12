import HoprEthereum from '.'
import { Hash } from './types'

import Debug from 'debug'
const log = Debug('hopr-core-ethereum:hashedSecret')

import { randomBytes, createHash } from 'crypto'
import { u8aEquals, u8aToHex, stringToU8a } from '@hoprnet/hopr-utils'

export const GIANT_STEP_WIDTH = 10000
export const TOTAL_ITERATIONS = 100000
class HashedSecret {
  public _onChainValuesInitialized: boolean

  constructor(private coreConnector: HoprEthereum) {
    this._onChainValuesInitialized = false
  }

  /**
   * generate and set account secret
   */
  async submit(nonce?: number): Promise<void> {
    const hashedSecret = await this.create()

    await this.coreConnector.utils.waitForConfirmation(
      (
        await this.coreConnector.signTransaction(
          this.coreConnector.hoprChannels.methods.setHashedSecret(u8aToHex(hashedSecret)),
          {
            from: (await this.coreConnector.account.address).toHex(),
            to: this.coreConnector.hoprChannels.options.address,
            nonce: nonce || (await this.coreConnector.account.nonce),
          }
        )
      ).send()
    )

    this._onChainValuesInitialized = true
  }

  /**
   * Checks whether node has an account secret set onchain and offchain
   * @returns a promise resolved true if secret is set correctly
   */
  async check(): Promise<void> {
    let [onChainSecret, offChainSecret] = await Promise.all([
      // get onChainSecret
      this.coreConnector.hoprChannels.methods
        .accounts((await this.coreConnector.account.address).toHex())
        .call()
        .then((res) => stringToU8a(res.hashedSecret))
        .then((secret: Uint8Array) => {
          if (u8aEquals(secret, new Uint8Array(this.coreConnector.types.Hash.SIZE).fill(0x00))) {
            return undefined
          }

          return new Hash(secret)
        }),
      // get offChainSecret
      this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecret())).catch((err) => {
        if (err.notFound != true) {
          throw err
        }
      }),
    ])

    let hasOffChainSecret = typeof offChainSecret !== 'undefined'
    let hasOnChainSecret = typeof onChainSecret !== 'undefined'

    if (hasOffChainSecret && hasOnChainSecret) {
      try {
        await this.getPreimage(onChainSecret)
      } catch (err) {
        throw err
      }
    } else if (hasOffChainSecret != hasOnChainSecret) {
      if (hasOffChainSecret) {
        log(`Key is present off-chain but not on-chain, submitting..`)
        let hashedSecret = await this.coreConnector.db.get(
          Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(TOTAL_ITERATIONS - GIANT_STEP_WIDTH))
        )
        for (let i = 0; i < GIANT_STEP_WIDTH; i++) {
          hashedSecret = await this.coreConnector.utils.hash(hashedSecret)
        }

        // @TODO this potentially dangerous because it increases the account counter
        await this.coreConnector.utils.waitForConfirmation(
          (
            await this.coreConnector.signTransaction(
              this.coreConnector.hoprChannels.methods.setHashedSecret(u8aToHex(hashedSecret)),
              {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address,
                nonce: await this.coreConnector.account.nonce,
              }
            )
          ).send()
        )
        hasOnChainSecret = true
      } else {
        log(`Key is present on-chain but not in our database.`)
        if (this.coreConnector.options.debug) {
          this.create()
          hasOffChainSecret = true
        }
      }
    }

    this._onChainValuesInitialized = hasOffChainSecret && hasOnChainSecret
  }

  /**
   * Returns a deterministic secret that is used in debug mode.
   */
  private getDebugAccountSecret(): Uint8Array {
    return createHash('sha256').update(this.coreConnector.account.keys.onChain.pubKey).digest()
  }

  /**
   * Creates the on-chain secret and stores the intermediate values
   * into the database.
   */
  async create(): Promise<Hash> {
    let onChainSecret = new Hash(
      this.coreConnector.options.debug ? this.getDebugAccountSecret() : randomBytes(Hash.SIZE)
    )
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

  /**
   * Tries to find a pre-image for the given hash by using the intermediate
   * values from the database.
   * @param hash the hash to find a preImage for
   */
  async getPreimage(hash: Hash): Promise<{ preImage: Hash; index: number }> {
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
