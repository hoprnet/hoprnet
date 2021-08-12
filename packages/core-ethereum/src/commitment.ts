import { iterateHash, recoverIteratedHash, HoprDB, Hash } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'
import Debug from 'debug'
import type { Receipt } from './ethereum'
import Indexer from './indexer'

const log = Debug('hopr-core-ethereum:commitment')

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000

function hashFunction(msg: Uint8Array): Uint8Array {
  return Hash.create(msg).serialize().slice(0, Hash.SIZE)
}

// See the specification for a full description of what this is for, but
// essentially we want to post a 'commitment' to each ticket, that allows
// later verification by giving a 'preimage' of that commitment.
//
// We need to persist this string of commitments in the database, and support
// syncing back and forth with those that have been persisted on chain.
export class Commitment {
  private initialized: boolean = false

  constructor(
    private setChainCommitment: (commitment: Hash) => Promise<Receipt>,
    private getChainCommitment: () => Promise<Hash>,
    private db: HoprDB,
    private channelId: Hash, // used in db key
    private indexer: Indexer
  ) {}

  public async getCurrentCommitment(): Promise<Hash> {
    if (!this.initialized) {
      await this.initialize()
    }
    return await this.db.getCurrentCommitment(this.channelId)
  }

  public async bumpCommitment() {
    if (!this.initialized) {
      await this.initialize()
    }

    await this.db.setCurrentCommitment(
      this.channelId,
      await this.findPreImage(await this.db.getCurrentCommitment(this.channelId))
    )
  }

  public async findPreImage(hash: Hash): Promise<Hash> {
    // TODO refactor after we move primitives
    let result = await recoverIteratedHash(
      hash.serialize(),
      hashFunction,
      this.searchDBFor.bind(this),
      TOTAL_ITERATIONS,
      DB_ITERATION_BLOCK_SIZE
    )
    if (result == undefined) {
      throw Error(`Could not find preImage. Searching for ${hash.toHex()}`)
    }
    return new Hash(Uint8Array.from(result.preImage))
  }

  public async initialize(): Promise<void> {
    if (this.initialized) return

    // @FIXME only works for first iteration
    const isWaiting = this.indexer.waitForCommitment(this.channelId)

    if (isWaiting != undefined) {
      log(`Found pending commitment, waiting until state appears in indexer`)

      await isWaiting

      log(`Commitment is set, commitment is now fully initialized.`)

      this.initialized = true
      return
    }

    const dbContains = await this.hasDBSecret()
    const chainCommitment = await this.getChainCommitment()
    if (chainCommitment && dbContains) {
      try {
        await this.findPreImage(chainCommitment) // throws if not found
        await this.db.getCurrentCommitment(this.channelId) // Find out if we have one
        this.initialized = true
        return
      } catch (e) {
        log(`Secret is found but failed to find preimage, reinitializing.. ${e.message}`)
      }
    }
    log(`reinitializing (db: ${dbContains}, chain: ${chainCommitment}})`)
    await this.createCommitmentChain()
    this.initialized = true
  }

  private async createCommitmentChain(): Promise<void> {
    const seed = new Hash(Uint8Array.from(randomBytes(Hash.SIZE))) // TODO seed off privKey + channel
    const result = await iterateHash(seed.serialize(), hashFunction, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE)
    await this.db.storeHashIntermediaries(this.channelId, result.intermediates)
    const current = new Hash(Uint8Array.from(result.hash))
    await this.db.setCurrentCommitment(this.channelId, current)
    await this.setChainCommitment(current)
    log('commitment chain initialized')
  }

  private async hasDBSecret(): Promise<boolean> {
    return (await this.db.getCommitment(this.channelId, 0)) != undefined
  }

  private async searchDBFor(iteration: number): Promise<Uint8Array | undefined> {
    return await this.db.getCommitment(this.channelId, iteration)
  }
}
