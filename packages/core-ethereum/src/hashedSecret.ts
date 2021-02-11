import type HoprEthereum from '.'
import { Hash } from './types'
import Debug from 'debug'
const log = Debug('hopr-core-ethereum:hashedSecret')
import { randomBytes } from 'crypto'
import { u8aToHex, u8aConcat, iterateHash, recoverIteratedHash } from '@hoprnet/hopr-utils'
import type { Intermediate } from '@hoprnet/hopr-utils'

import { publicKeyConvert } from 'secp256k1'
import { hash as hashFunction } from './utils'

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000
export const HASHED_SECRET_WIDTH = 27


const isNullAccount = (a: string) => a == null || ['0', '0x', '0x'.padEnd(66, '0')].includes(a)

class HashedSecret {
  private initialized: boolean
  private onChainSecret: Hash
  private offChainSecret: Hash

  constructor(private coreConnector: HoprEthereum) {
    this.initialized = false
  }

  /**
   * @returns a promise that resolves to a Hash if secret is found
   */
  private async getOffChainSecret(): Promise<Hash | undefined> {
    try {
      return await this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecret()))
    } catch (err) {
      if (!err.notFound) {
        throw err
      }
      return
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

  private async hint(index: number): Promise<Uint8Array | undefined> {
    try {
      return await this.coreConnector.db.get(Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(index)))
    } catch (err) {
      if (err.notFound) {
        return
      }
      throw err
    }
  }
  /**
   * Creates a random secret OR a deterministic one if running in debug mode,
   * it will then loop X amount of times, on each loop we hash the previous result.
   * We store the last result.
   * @returns a promise that resolves to a Hash if secret is found
   */
  private async createAndStoreSecretOffChain(debug: boolean): Promise<Hash> {
    let onChainSecret = debug ? await this.getDebugAccountSecret() : new Hash(randomBytes(HASHED_SECRET_WIDTH))

    let dbBatch = this.coreConnector.db.batch()

    const result = await iterateHash(
      onChainSecret,
      hashFunction,
      TOTAL_ITERATIONS,
      DB_ITERATION_BLOCK_SIZE
    )

    for (const intermediate of result.intermediates) {
      dbBatch = dbBatch.put(
        Buffer.from(this.coreConnector.dbKeys.OnChainSecretIntermediary(intermediate.iteration)),
        Buffer.from(intermediate.preImage)
      )
    }

    await dbBatch.write()

    return new Hash(result.hash)
  }

  private async storeSecretOnChain(secret: Hash): Promise<void> {
    log(`storing secret on chain, setting secret to ${u8aToHex(secret)}`)
    const account = await this.coreConnector.hoprChannels.methods
      .accounts((await this.coreConnector.account.address).toHex())
      .call()

    if (isNullAccount(account.accountX)) {
      const uncompressedPubKey = publicKeyConvert(this.coreConnector.account.keys.onChain.pubKey, false).slice(1)
      log('account is also null, calling channel.init')
      try {
        await this.coreConnector.utils.waitForConfirmation(
          (
            await this.coreConnector.account.signTransaction(
              {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address
              },
              this.coreConnector.hoprChannels.methods.init(
                u8aToHex(uncompressedPubKey.slice(0, 32)),
                u8aToHex(uncompressedPubKey.slice(32, 64)),
                u8aToHex(secret)
              )
            )
          ).send()
        )
      } catch (e) {
        if (e.message.match(/Account must not be set/)) {
          // There is a potential race condition due to the fact that 2 init
          // calls may be in flight at once, and therefore we may have init
          // called on an initialized account. If so, trying again should solve
          // the problem.
          log('race condition encountered in HoprChannel.init - retrying')
          return this.storeSecretOnChain(secret)
        }
        throw e
      }
    } else {
      // @TODO this is potentially dangerous because it increases the account counter
      log('account is already on chain, storing secret.')
      try {
        await this.coreConnector.utils.waitForConfirmation(
          (
            await this.coreConnector.account.signTransaction(
              {
                from: (await this.coreConnector.account.address).toHex(),
                to: this.coreConnector.hoprChannels.options.address
              },
              this.coreConnector.hoprChannels.methods.setHashedSecret(u8aToHex(secret))
            )
          ).send()
        )
      } catch (e) {
        if (e.message.match(/new and old hashedSecrets are the same/)) {
          // NBD. no-op
          return
        }
        throw e
      }
    }

    log('stored on chain')
  }

  private async calcOnChainSecretFromDb(debug?: boolean): Promise<Hash | never> {
    let result = await iterateHash(
      debug == true ? await this.getDebugAccountSecret() : undefined,
      hashFunction,
      TOTAL_ITERATIONS,
      DB_ITERATION_BLOCK_SIZE,
      this.hint.bind(this)
    )

    if (result == undefined) {
      return await this.createAndStoreSecretOffChain(debug)
    }

    return new Hash(result.hash)
  }

  /**
   * Tries to find a pre-image for the given hash by using the intermediate
   * values from the database.
   * @param hash the hash to find a preImage for
   */
  public async findPreImage(hash: Uint8Array): Promise<Intermediate> {
    if (hash.length != HASHED_SECRET_WIDTH) {
      throw Error(
        `Invalid length. Expected a Uint8Array with ${HASHED_SECRET_WIDTH} elements but got one with ${hash.length}`
      )
    }

    let result = await recoverIteratedHash(
      hash,
      hashFunction,
      this.hint.bind(this),
      TOTAL_ITERATIONS,
      DB_ITERATION_BLOCK_SIZE
    )

    if (result == undefined) {
      throw Error(`Could not find preImage.`)
    }

    return result
  }

  /**
   * Initializes hashedSecret.
   */
  public async initialize(debug?: boolean): Promise<void> {
    if (this.initialized) return

    this.offChainSecret = await this.getOffChainSecret()
    this.onChainSecret = await this.coreConnector.account.onChainSecret
    if (this.onChainSecret != undefined && this.offChainSecret != undefined) {
      try {
        await this.findPreImage(this.onChainSecret) // throws if not found
        this.initialized = true
        return 
      } catch (_e) {
        log(`Secret is found but failed to find preimage, reinitializing..`)
      } 
    }
    log(`Secret is not initialized.`)
    if (this.offChainSecret && !this.onChainSecret){
      log('initializing for the first time')
      const onChainSecret = await this.calcOnChainSecretFromDb(debug)
      await this.storeSecretOnChain(onChainSecret)
    } else {
      log('reinitializing')
      const offChainSecret = await this.createAndStoreSecretOffChain(debug)
      log('... secret generated, storing')
      await this.storeSecretOnChain(offChainSecret)
      log('... initialized')
    }
    this.initialized = true
  }
}

export default HashedSecret
