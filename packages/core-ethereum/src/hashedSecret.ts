import { Hash, AcknowledgedTicket, Ticket, Balance, SignedTicket, AccountId, TicketEpoch } from './types'
import Debug from 'debug'
import { randomBytes } from 'crypto'
import { u8aToHex, u8aConcat, iterateHash, recoverIteratedHash, u8aLessThanOrEqual } from '@hoprnet/hopr-utils'
import { stringToU8a, u8aIsEmpty, u8aCompare } from '@hoprnet/hopr-utils'
import { publicKeyConvert } from 'secp256k1'
import { hash, waitForConfirmation, computeWinningProbability } from './utils'
import { OnChainSecret, OnChainSecretIntermediary } from './dbKeys'
import type { LevelUp } from 'levelup'
import type { HoprChannels } from './tsc/web3/HoprChannels'
import type Account from './account'

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000
export const HASHED_SECRET_WIDTH = 27

const log = Debug('hopr-core-ethereum:probabilisticPayments')
const isNullAccount = (a: string) => a == null || ['0', '0x', '0x'.padEnd(66, '0')].includes(a)

/**
 * Decides whether a ticket is a win or not.
 * Note that this mimics the on-chain logic.
 * @dev Purpose of the function is to check the validity of
 * a ticket before we submit it to the blockchain.
 * @param ticketHash hash value of the ticket to check
 * @param challengeResponse response that solves the signed challenge
 * @param preImage preImage of the current onChainSecret
 * @param winProb winning probability of the ticket
 */
async function isWinningTicket(ticketHash: Hash, challengeResponse: Hash, preImage: Hash, winProb: Uint8Array) {
  console.log(
    await hash(u8aConcat(ticketHash, preImage, challengeResponse)),
    winProb,
    u8aCompare(await hash(u8aConcat(ticketHash, preImage, challengeResponse)), winProb)
  )
  return u8aLessThanOrEqual(await hash(u8aConcat(ticketHash, preImage, challengeResponse)), winProb)
}

export async function hashFunction(msg: Uint8Array): Promise<Uint8Array> {
  return (await hash(msg)).slice(0, HASHED_SECRET_WIDTH)
}

async function getFromDB<T>(db: LevelUp, key): Promise<T | undefined> {
  try {
    return await db.get(Buffer.from(key))
  } catch (err) {
    if (!err.notFound) {
      throw err
    }
    return
  }
}

export class ProbabilisticPayments {
  private initialized: boolean = false
  private onChainSecret: Hash
  private offChainSecret: Hash
  private currentPreImage: Uint8Array

  constructor(private db: LevelUp, private account: Account, private channels: HoprChannels) {}

  /**
   * @returns a deterministic secret that is used in debug mode
   */
  private async getDebugAccountSecret(): Promise<Hash> {
    const account = await this.channels.methods.accounts((await this.account.address).toHex()).call()
    return new Hash(
      await hashFunction(u8aConcat(new Uint8Array([parseInt(account.counter)]), this.account.keys.onChain.pubKey))
    )
  }

  /**
   * Creates a random secret OR a deterministic one if running in debug mode,
   * it will then loop X amount of times, on each loop we hash the previous result.
   * We store the last result.
   * @returns a promise that resolves to the onChainSecret
   */
  private async createAndStoreSecretOffChainAndReturnOnChainSecret(debug: boolean): Promise<Hash> {
    this.offChainSecret = debug ? await this.getDebugAccountSecret() : new Hash(randomBytes(HASHED_SECRET_WIDTH))
    let dbBatch = this.db.batch()
    const hashes = await iterateHash(this.offChainSecret, hashFunction, TOTAL_ITERATIONS)
    for (let i = 0; i <= TOTAL_ITERATIONS; i += DB_ITERATION_BLOCK_SIZE) {
      log('storing intermediate', i)
      dbBatch = dbBatch.put(Buffer.from(OnChainSecretIntermediary(i)), Buffer.from(hashes[i]))
    }
    await dbBatch.write()
    return new Hash(hashes[hashes.length - 1])
  }

  private async storeSecretOnChain(secret: Hash): Promise<void> {
    log(`storing secret on chain, setting secret to ${u8aToHex(secret)}`)
    const address = (await this.account.address).toHex()
    const account = await this.channels.methods.accounts(address).call()

    if (isNullAccount(account.accountX)) {
      const uncompressedPubKey = publicKeyConvert(this.account.keys.onChain.pubKey, false).slice(1)
      log('account is also null, calling channel.init')
      try {
        await waitForConfirmation(
          (
            await this.account.signTransaction(
              {
                from: address,
                to: this.channels.options.address
              },
              this.channels.methods.init(
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
        await waitForConfirmation(
          (
            await this.account.signTransaction(
              {
                from: address,
                to: this.channels.options.address
              },
              this.channels.methods.setHashedSecret(u8aToHex(secret))
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
    const start = debug ? await this.getDebugAccountSecret() : this.offChainSecret
    let hashes = await iterateHash(start, hashFunction, TOTAL_ITERATIONS)
    return new Hash(hashes[hashes.length - 1])
  }

  /**
   * Tries to find a pre-image for the given hash by using the intermediate
   * values from the database.
   * @param hash the hash to find a preImage for
   */
  public async findPreImage(hash: Uint8Array): Promise<Uint8Array> {
    // TODO only public for test, make private
    if (hash.length != HASHED_SECRET_WIDTH) {
      throw Error(
        `Invalid length. Expected a Uint8Array with ${HASHED_SECRET_WIDTH} elements but got one with ${hash.length}`
      )
    }

    return await recoverIteratedHash(
      hash,
      hashFunction,
      (index) => getFromDB(this.db, OnChainSecretIntermediary(index)),
      TOTAL_ITERATIONS,
      DB_ITERATION_BLOCK_SIZE
    )
  }

  private async findOnChainSecret() {
    const res = await this.channels.methods.accounts((await this.account.address).toHex()).call()
    const hashedSecret = stringToU8a(res.hashedSecret)
    if (u8aIsEmpty(hashedSecret)) {
      return undefined
    }
    return new Hash(hashedSecret)
  }

  public async initialize(debug?: boolean): Promise<void> {
    if (this.initialized) return
    this.offChainSecret = await getFromDB(this.db, OnChainSecret())
    this.onChainSecret = await this.findOnChainSecret()
    if (this.onChainSecret && this.offChainSecret) {
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
      this.onChainSecret = await this.calcOnChainSecretFromDb(debug)
      await this.storeSecretOnChain(this.onChainSecret)
    } else {
      log('reinitializing')
      this.onChainSecret = await this.createAndStoreSecretOffChainAndReturnOnChainSecret(debug)
      await this.storeSecretOnChain(this.onChainSecret)
    }
    this.currentPreImage = await this.findPreImage(this.onChainSecret) //TODO
    this.initialized = true
  }

  // When the secret changes on chain, we need to update
  public updateOnChainSecret(secret: Hash) {
    this.onChainSecret = secret
    // TODO update db
  }

  public getOnChainSecret() {
    return this.onChainSecret
  }

  public async validateTicket(ticket: AcknowledgedTicket): Promise<boolean> {
    const s = await ticket.signedTicket
    log('validate')
    if (await isWinningTicket(await s.ticket.hash, ticket.response, ticket.preImage, s.ticket.winProb)) {
      ticket.preImage = new Hash(this.currentPreImage)
      this.currentPreImage = await this.findPreImage(this.currentPreImage)
      return true
    }
    log('>> invalid')
    return false
  }

  public async issueTicket(
    amount: Balance,
    counterparty: AccountId,
    challenge: Hash,
    epoch: TicketEpoch,
    channelIteration: TicketEpoch,
    winProb: number
  ): Promise<SignedTicket> {
    const ticketWinProb = new Hash(computeWinningProbability(winProb))
    const signedTicket = new SignedTicket()
    const ticket = new Ticket(
      {
        bytes: signedTicket.buffer,
        offset: signedTicket.ticketOffset
      },
      {
        counterparty,
        challenge,
        epoch,
        amount,
        winProb: ticketWinProb,
        channelIteration
      }
    )

    await ticket.sign(this.account.keys.onChain.privKey, undefined, {
      bytes: signedTicket.buffer,
      offset: signedTicket.signatureOffset
    })

    return signedTicket
  }
}
