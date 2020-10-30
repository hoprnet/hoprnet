import type HoprEthereum from '.'
import { isWinningTicket } from './utils'
import { Hash } from './types'

import Debug from 'debug'
const log = Debug('hopr-core-ethereum:hashedSecret')

import { randomBytes } from 'crypto'
import { u8aEquals, u8aToHex, u8aConcat } from '@hoprnet/hopr-utils'
import { publicKeyConvert } from 'secp256k1'
import { AcknowledgedTicket } from './types'

export const GIANT_STEP_WIDTH = 10000
export const TOTAL_ITERATIONS = 100000
export const HASHED_SECRET_WIDTH = 27

export type PreImageResult = {
  preImage: Hash
  index: number
}

class HashedSecret {
  constructor(private coreConnector: HoprEthereum) {}

  /**
   * @returns a promise that resolves to a Hash if secret is found
   */
  private async getOnChainSecret(): Promise<Hash | void> {
    return this.coreConnector.account.onChainSecret
  }

  /**
   * @returns a promise that resolves to a Hash if secret is found
   */
  private async getOffChainSecret(): Promise<Hash | undefined> {
    try {
      return await this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecret())) as any
    } catch {
      return undefined
    }
  }

  /**
   * @returns a deterministic secret that is used in debug mode
   */
  private async getDebugAccountSecret(): Promise<Hash> {
    const account = await this.coreConnector.hoprChannels.methods
      .accounts((await this.coreConnector.account.address).toHex())
      .call()

    return new Hash(
      (
        await this.coreConnector.utils.hash(
          u8aConcat(new Uint8Array([parseInt(account.counter)]), this.coreConnector.account.keys.onChain.pubKey)
        )
      ).slice(0, HASHED_SECRET_WIDTH)
    )
  }

  /**
   * Creates a random secret OR a deterministic one if running in debug mode,
   * it will then loop X amount of times, on each loop we hash the previous result.
   * We store the last result.
   * @returns a promise that resolves to a Hash if secret is found
   */
  private async createAndStoreSecretOffChain(): Promise<Hash> {
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
   * Store secret onchain.
   */
  private async storeSecretOnChain(secret: Hash, nonce?: number): Promise<void> {
    const account = await this.coreConnector.hoprChannels.methods
      .accounts((await this.coreConnector.account.address).toHex())
      .call()

    const _nonce = nonce ? nonce : await this.coreConnector.account.nonce

    if (account.accountX == null || ['0', '0x', '0x'.padEnd(66, '0')].includes(account.accountX)) {
      const uncompressedPubKey = publicKeyConvert(this.coreConnector.account.keys.onChain.pubKey, false).slice(1)

      await this.coreConnector.utils.waitForConfirmation(
        (
          await this.coreConnector.signTransaction(
            {
              from: (await this.coreConnector.account.address).toHex(),
              to: this.coreConnector.hoprChannels.options.address,
              nonce: _nonce
            },
            this.coreConnector.hoprChannels.methods.init(
              u8aToHex(uncompressedPubKey.slice(0, 32)),
              u8aToHex(uncompressedPubKey.slice(32, 64)),
              u8aToHex(secret)
            )
          )
        ).send()
      )
    } else {
      // @TODO this is potentially dangerous because it increases the account counter
      await this.coreConnector.utils.waitForConfirmation(
        (
          await this.coreConnector.signTransaction(
            {
              from: (await this.coreConnector.account.address).toHex(),
              to: this.coreConnector.hoprChannels.options.address,
              nonce: _nonce
            },
            this.coreConnector.hoprChannels.methods.setHashedSecret(u8aToHex(secret))
          )
        ).send()
      )
    }
  }

  private async calcOnChainSecretFromDb(): Promise<Hash> {
    let closestIntermediary = TOTAL_ITERATIONS - GIANT_STEP_WIDTH

    let intermediary: Uint8Array
    while (closestIntermediary > 0) {
      try {
        intermediary = await this.coreConnector.db.get(
          Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(closestIntermediary))
        ) as Uint8Array
        break
      } catch (err) {
        if (!err.notFound) {
          throw err
        }
        closestIntermediary -= GIANT_STEP_WIDTH
      }
    }

    if (closestIntermediary == 0) {
      try {
        intermediary = await this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecret())) as Uint8Array
      } catch (err) {
        if (!err.notFound) {
          throw err
        }

        return this.createAndStoreSecretOffChain()
      }
    }

    for (let i = 0; i < TOTAL_ITERATIONS - closestIntermediary; i++) {
      intermediary = (await this.coreConnector.utils.hash(intermediary)).slice(0, HASHED_SECRET_WIDTH)
    }

    return new Hash(intermediary)
  }

  /**
   * Tries to find a pre-image for the given hash by using the intermediate
   * values from the database.
   * @param hash the hash to find a preImage for
   */
  public async findPreImage(hash: Uint8Array): Promise<PreImageResult> {
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
          ) as Uint8Array
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
      throw Error('Preimage not found')
    }

    return { preImage: new Hash(intermediary), index }
  }

  /**
   * Check whether our secret exists and matches our onchain secret.
   * Both onChain and offChain secret must be present and matching.
   * @returns a promise that resolves to a true if everything is ok
   */
  public async check(): Promise<{
    initialized: boolean
    onChainSecret?: Hash
    offChainSecret?: Hash
  }> {
    const [onChainSecret, offChainSecret] = await Promise.all([this.getOnChainSecret(), this.getOffChainSecret()])
    const bothEmpty = typeof onChainSecret === 'undefined' && typeof offChainSecret === 'undefined'
    const bothExist = !bothEmpty && typeof onChainSecret !== 'undefined' && typeof offChainSecret !== 'undefined'

    // both are empty
    if (bothEmpty) {
      return { initialized: false }
    }
    // both exist
    else if (bothExist) {
      try {
        await this.findPreImage(onChainSecret)
        return { initialized: true, onChainSecret, offChainSecret }
      } catch (err) {
        log(err)
        return { initialized: false, onChainSecret, offChainSecret }
      }
    }
    // only one of them exists
    else {
      return { initialized: false, onChainSecret, offChainSecret }
    }
  }

  /**
   * Initializes hashedSecret.
   */
  public async initialize(nonce?: number): Promise<void> {
    const { initialized, onChainSecret, offChainSecret } = await this.check()
    if (initialized) {
      log(`Secret is initialized.`)
      return
    }

    const bothEmpty = typeof onChainSecret === 'undefined' && typeof offChainSecret === 'undefined'
    const bothExist = !bothEmpty && typeof onChainSecret !== 'undefined' && typeof offChainSecret !== 'undefined'
    const onlyOnChain = !bothEmpty && !bothExist && typeof onChainSecret !== 'undefined'

    if (bothEmpty) {
      log(`Secret not found, initializing..`)
    } else if (bothExist) {
      log(`Secret is found but failed to find preimage, reinitializing..`)
    } else if (onlyOnChain) {
      log(`Secret is present on-chain but not off-chain, reinitializing..`)
    } else {
      log(`Secret is present off-chain but not on-chain, submitting..`)
    }

    if (bothEmpty || bothExist || onlyOnChain) {
      const offChainSecret = await this.createAndStoreSecretOffChain()
      await this.storeSecretOnChain(offChainSecret, nonce)
    } else {
      const onChainSecret = await this.calcOnChainSecretFromDb()
      await this.storeSecretOnChain(onChainSecret, nonce)
    }
  }
}

export default HashedSecret
