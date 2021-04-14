import type { LevelUp } from 'levelup'
import type Account from './account'
import type { HoprChannels } from './contracts'
import { Hash } from './types'
import Debug from 'debug'
import { randomBytes } from 'crypto'
import { u8aConcat, iterateHash, recoverIteratedHash } from '@hoprnet/hopr-utils'
import { storeHashIntermediaries, getOnChainSecretIntermediary, getOnChainSecret } from './dbKeys'

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000

const log = Debug('hopr-core-ethereum:hashedSecret')

export async function hashFunction(msg: Uint8Array): Promise<Uint8Array> {
  return Hash.create(msg).serialize().slice(0, Hash.SIZE)
}

export class TicketCommitment {
  constructor() {}

  reserveCommitment() {}
}

class HashedSecret {
  private initialized: boolean = false
  private onChainSecret: Hash
  private offChainSecret: Hash

  constructor(private db: LevelUp, private account: Account, private channels: HoprChannels) {}

  /**
   * @returns a deterministic secret that is used in debug mode
   */
  private async getDebugAccountSecret(): Promise<Hash> {
    const account = await this.channels.accounts(this.account.address.toHex())
    return new Hash(
      await hashFunction(u8aConcat(new Uint8Array([account.counter.toNumber()]), this.account.publicKey.serialize()))
    )
  }

  /**
   * Creates a random secret OR a deterministic one if running in debug mode,
   * it will then loop X amount of times, on each loop we hash the previous result.
   * We store the last result.
   * @returns a promise that resolves to the onChainSecret
   */
  private async createAndStoreSecretOffChainAndReturnOnChainSecret(debug: boolean): Promise<Hash> {
    let onChainSecret = debug ? await this.getDebugAccountSecret() : new Hash(randomBytes(Hash.SIZE))
    const result = await iterateHash(onChainSecret.serialize(), hashFunction, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE)
    storeHashIntermediaries(this.db, result.intermediates)
    return new Hash(result.hash)
  }

  private async storeSecretOnChain(secret: Hash): Promise<void> {
    log(`storing secret on chain, setting secret to ${secret.toHex()}`)
    const address = this.account.address.toHex()
    const account = await this.channels.accounts(address)
    // has no secret stored onchain
    if (Number(account.counter) === 0) {
      log('account is also null, calling channel.init')
      try {
        const transaction = await this.account.sendTransaction(
          this.channels.initializeAccount,
          this.account.publicKey.toUncompressedPubKeyHex(),
          secret.toHex()
        )
        await transaction.wait()
        this.account.updateLocalState(secret)
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
        const transaction = await this.account.sendTransaction(this.channels.updateAccountSecret, secret.toHex())
        await transaction.wait()
        this.account.updateLocalState(secret)
      } catch (e) {
        if (e.message.match(/secret must not be the same as before/)) {
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
      debug == true ? (await this.getDebugAccountSecret()).serialize() : undefined,
      hashFunction,
      TOTAL_ITERATIONS,
      DB_ITERATION_BLOCK_SIZE,
      (x) => getOnChainSecretIntermediary(this.db, x)
    )

    if (result == undefined) {
      return await this.createAndStoreSecretOffChainAndReturnOnChainSecret(debug)
    }

    return new Hash(result.hash)
  }

  /**
   * Tries to find a pre-image for the given hash by using the intermediate
   * values from the database.
   * @param hash the hash to find a preImage for
   */
  public async findPreImage(hash: Hash): Promise<Hash> {
    let result = await recoverIteratedHash(
      hash.serialize(),
      hashFunction,
      (x) => getOnChainSecretIntermediary(this.db, x),
      TOTAL_ITERATIONS,
      DB_ITERATION_BLOCK_SIZE
    )
    if (result == undefined) {
      throw Error(`Could not find preImage.`)
    }
    return new Hash(result.preImage)
  }

  public async initialize(debug?: boolean): Promise<void> {
    if (this.initialized) return
    this.offChainSecret = await getOnChainSecret(this.db)
    this.onChainSecret = await this.account.getOnChainSecret()
    if (this.onChainSecret != undefined && this.offChainSecret != undefined) {
      try {
        await this.findPreImage(this.onChainSecret) // throws if not found
        this.initialized = true
        return
      } catch (_e) {
        log(`Secret is found but failed to find preimage, reinitializing..`)
      }
    }
    if (this.offChainSecret && !this.onChainSecret) {
      log('secret exists offchain but not on chain')
      const onChainSecret = await this.calcOnChainSecretFromDb(debug)
      await this.storeSecretOnChain(onChainSecret)
    } else {
      log('reinitializing')
      const onChainSecret = await this.createAndStoreSecretOffChainAndReturnOnChainSecret(debug)
      await this.storeSecretOnChain(onChainSecret)
    }
    this.initialized = true
  }
}

export default HashedSecret
