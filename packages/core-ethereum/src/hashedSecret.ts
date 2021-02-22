import { Hash, AcknowledgedTicket, Ticket, Balance, SignedTicket, AccountId, TicketEpoch } from './types'
import Debug from 'debug'
import { randomBytes } from 'crypto'
import { u8aToHex, u8aConcat, iterateHash, recoverIteratedHash, u8aLessThanOrEqual } from '@hoprnet/hopr-utils'
import { hash, computeWinningProbability } from './utils'
import { OnChainSecret, OnChainSecretIntermediary } from './dbKeys'
import type { LevelUp } from 'levelup'
import type { ValidateResponse, RedeemStatus } from '@hoprnet/hopr-core-connector-interface'
import { checkChallenge } from './utils'
import { u8aCompare } from '@hoprnet/hopr-utils'

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000
export const HASHED_SECRET_WIDTH = 27

const log = Debug('hopr-core-ethereum:probabilisticPayments')

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
async function isWinningTicket(ticketHash: Hash, challengeResponse: Hash, preImage: Uint8Array, winProb: Uint8Array) {
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

  constructor(
    private db: LevelUp,
    private privKey: Uint8Array,
    private storeSecretOnChain: (hash: Hash) => Promise<void>,
    private findOnChainSecret: () => Promise<Hash | undefined>,
    private submitTicketRedemption: (ackTicket: AcknowledgedTicket) => Promise<void>
  ) {}

  /**
   * @returns a deterministic secret that is used in debug mode
   */
  /*
  private async getDebugAccountSecret(): Promise<Hash> {
    const account = await this.channels.methods.accounts((await this.account.address).toHex()).call()
    return new Hash(
      await hashFunction(u8aConcat(new Uint8Array([parseInt(account.counter)]), this.account.keys.onChain.pubKey))
    )
  }
*/
  /**
   * Creates a random secret OR a deterministic one if running in debug mode,
   * it will then loop X amount of times, on each loop we hash the previous result.
   * We store the last result.
   * @returns a promise that resolves to the onChainSecret
   */
  private async createAndStoreSecretOffChainAndReturnOnChainSecret(_debug: boolean): Promise<Hash> {
    this.offChainSecret = /*debug ? await this.getDebugAccountSecret() :*/ new Hash(randomBytes(HASHED_SECRET_WIDTH))
    let dbBatch = this.db.batch()
    const hashes = await iterateHash(this.offChainSecret, hashFunction, TOTAL_ITERATIONS)
    for (let i = 0; i <= TOTAL_ITERATIONS; i += DB_ITERATION_BLOCK_SIZE) {
      log('storing intermediate', i)
      dbBatch = dbBatch.put(Buffer.from(OnChainSecretIntermediary(i)), Buffer.from(hashes[i]))
    }
    await dbBatch.write()
    return new Hash(hashes[hashes.length - 1])
  }

  private async calcOnChainSecretFromDb(_debug?: boolean): Promise<Hash | never> {
    const start = /*debug ? await this.getDebugAccountSecret() :*/ this.offChainSecret
    let hashes = await iterateHash(start, hashFunction, TOTAL_ITERATIONS)
    return new Hash(hashes[hashes.length - 1])
  }

  /**
   * Tries to find a pre-image for the given hash by using the intermediate
   * values from the database.
   * @param hash the hash to find a preImage for
   */
  private async findPreImage(hash: Uint8Array): Promise<Uint8Array> {
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

  public async __test_isValidIteratedHash(hash: Uint8Array): Promise<boolean> {
    try {
      await this.findPreImage(hash)
      log('found pre image')
      return true
    } catch (e) {
      log('preimage not found', e)
      return false
    }
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

  /*
   * Take a signed ticket and transform it into an acknowledged ticket if it's a
   * winning ticket, or undefined if it's not.
   */
  public async validateTicket(ticket: SignedTicket, response: Hash): Promise<ValidateResponse> {
    log('validate')

    const validChallenge = await checkChallenge(ticket.ticket.challenge, response)
    if (!validChallenge) {
      log(`Failed to submit ticket ${u8aToHex(ticket.ticket.challenge)}: E_CHALLENGE`)
      return { status: 'E_CHALLENGE' }
    }

    if (await isWinningTicket(await ticket.ticket.hash, response, this.currentPreImage, ticket.ticket.winProb)) {
      this.currentPreImage = await this.findPreImage(this.currentPreImage)
      return { status: 'SUCCESS', ticket: new AcknowledgedTicket(ticket, response, new Hash(this.currentPreImage)) }
    }
    log('>> invalid')
    return { status: 'E_TICKET_FAILED' }
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

    await ticket.sign(this.privKey, undefined, {
      bytes: signedTicket.buffer,
      offset: signedTicket.signatureOffset
    })

    return signedTicket
  }

  public async redeemTicket(ackTicket: AcknowledgedTicket): Promise<RedeemStatus> {
    try {
      await this.submitTicketRedemption(ackTicket)
      this.updateOnChainSecret(ackTicket.getPreImage()) // redemption contract updates on chain
      log('Successfully submitted ticket')
      return { status: 'SUCCESS' }
    } catch (err) {
      // TODO - check if it's E_NO_GAS
      log('Unexpected error when submitting ticket', err)
      throw err
    }
  }
}
