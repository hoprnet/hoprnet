import HoprEthereum from '.'
import { Hash } from './types'

import Debug from 'debug'
const log = Debug('hopr-core-ethereum:hashedSecret')

import { randomBytes } from 'crypto'
import { u8aEquals, u8aToHex, stringToU8a, u8aConcat } from '@hoprnet/hopr-utils'
import { publicKeyConvert } from 'secp256k1'

export const GIANT_STEP_WIDTH = 10000
export const TOTAL_ITERATIONS = 100000

export const HASHED_SECRET_WIDTH = 27

class HashedSecret {
  public _onChainValuesInitialized: boolean

  constructor(private coreConnector: HoprEthereum) {
    this._onChainValuesInitialized = false
  }

  async submitFromDatabase(nonce?: number) {
    log(`Key is present off-chain but not on-chain, submitting..`)
    let hashedSecret: Uint8Array = await this.coreConnector.db.get(
      Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(TOTAL_ITERATIONS - GIANT_STEP_WIDTH))
    )
    for (let i = 0; i < GIANT_STEP_WIDTH; i++) {
      hashedSecret = await this.coreConnector.utils.hash(hashedSecret.slice(0, HASHED_SECRET_WIDTH))
    }

    await this._submit(hashedSecret, nonce)
  }
  /**
   * generate and set account secret
   */
  async submit(nonce?: number): Promise<void> {
    await this._submit(await this.create(), nonce)

    this._onChainValuesInitialized = true
  }

  private async _submit(hashedSecret: Uint8Array, nonce?: number) {
    const account = await this.coreConnector.hoprChannels.methods
      .accounts((await this.coreConnector.account.address).toHex())
      .call()

    if (account.accountX == null || ['0', '0x', '0x'.padEnd(66, '0')].includes(account.accountX)) {
      const uncompressedPubKey = publicKeyConvert(this.coreConnector.account.keys.onChain.pubKey, false).slice(1)

      await this.coreConnector.utils.waitForConfirmation(
        (
          await this.coreConnector.signTransaction(
            this.coreConnector.hoprChannels.methods.init(
              u8aToHex(uncompressedPubKey.slice(0, 32)),
              u8aToHex(uncompressedPubKey.slice(32, 64)),
              u8aToHex(hashedSecret)
            ),
            {
              from: (await this.coreConnector.account.address).toHex(),
              to: this.coreConnector.hoprChannels.options.address,
              nonce: nonce || (await this.coreConnector.account.nonce),
            }
          )
        ).send()
      )
    } else {
      // @TODO this is potentially dangerous because it increases the account counter
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
    }
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
        .then((res) => {
          const hashedSecret = stringToU8a(res.hashedSecret)
          if (u8aEquals(hashedSecret, new Uint8Array(HASHED_SECRET_WIDTH).fill(0x00))) {
            return undefined
          }

          return new Hash(hashedSecret)
        }),
      // get offChainSecret
      this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecret())).catch((err) => {
        if (err.notFound != true) {
          throw err
        }
      }),
    ])

    let hasOffChainSecret = offChainSecret != null
    let hasOnChainSecret = onChainSecret != null

    if (hasOffChainSecret && hasOnChainSecret) {
      // make sure that we are able to recover the pre-image
      await this.getPreimage(onChainSecret)
    } else if (hasOffChainSecret != hasOnChainSecret) {
      if (hasOffChainSecret) {
        await this.submitFromDatabase()
        hasOnChainSecret = true
      } else {
        log(`Key is present on-chain but not in our database.`)
        if (this.coreConnector.options.debug) {
          log(`DEBUG mode: Writing debug secret to database`)
          await this.create()
          hasOffChainSecret = true
        }
      }
    }

    this._onChainValuesInitialized = hasOffChainSecret && hasOnChainSecret
  }

  /**
   * Returns a deterministic secret that is used in debug mode.
   */
  private async getDebugAccountSecret(): Promise<Uint8Array> {
    const account = await this.coreConnector.hoprChannels.methods
      .accounts((await this.coreConnector.account.address).toHex())
      .call()

    return (
      await this.coreConnector.utils.hash(
        u8aConcat(new Uint8Array([parseInt(account.counter)]), this.coreConnector.account.keys.onChain.pubKey)
      )
    ).slice(0, HASHED_SECRET_WIDTH)
  }

  /**
   * Creates the on-chain secret and stores the intermediate values
   * into the database.
   */
  async create(): Promise<Uint8Array> {
    let onChainSecret = this.coreConnector.options.debug
      ? await this.getDebugAccountSecret()
      : new Hash(randomBytes(HASHED_SECRET_WIDTH))

    let onChainSecretIntermediary = onChainSecret
    let dbBatch = this.coreConnector.db.batch()

    for (let i = 0; i < TOTAL_ITERATIONS; i++) {
      if (i % GIANT_STEP_WIDTH == 0) {
        dbBatch = dbBatch.put(
          Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(i)),
          Buffer.from(onChainSecretIntermediary)
        )
      }
      onChainSecretIntermediary = new Hash(
        (await this.coreConnector.utils.hash(onChainSecretIntermediary)).slice(0, HASHED_SECRET_WIDTH)
      )
    }

    await dbBatch.write()

    return onChainSecretIntermediary
  }

  /**
   * Tries to find a pre-image for the given hash by using the intermediate
   * values from the database.
   * @param hash the hash to find a preImage for
   */
  async getPreimage(hash: Uint8Array): Promise<{ preImage: Hash; index: number }> {
    if (hash.length != HASHED_SECRET_WIDTH) {
      throw Error(
        `Invalid length. Expected a Uint8Array with ${HASHED_SECRET_WIDTH} elements but got one with ${hash.length}`
      )
    }

    let closestIntermediary = TOTAL_ITERATIONS - GIANT_STEP_WIDTH
    let intermediary: Uint8Array
    let upperBound = TOTAL_ITERATIONS

    let hashedIntermediary: Uint8Array
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
        hashedIntermediary = (await this.coreConnector.utils.hash(intermediary)).slice(0, HASHED_SECRET_WIDTH)
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

    return { preImage: new Hash(intermediary), index }
  }
}

export default HashedSecret
