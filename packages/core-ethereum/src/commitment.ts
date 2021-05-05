import { iterateHash, recoverIteratedHash, HoprDB, Hash } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'
import Debug from 'debug'

const log = Debug('hopr-core-ethereum:commitment')

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000

async function hashFunction(msg: Uint8Array): Promise<Uint8Array> {
  return Hash.create(msg).serialize().slice(0, Hash.SIZE)
}

export class Commitment {
  private initialized: boolean = false

  constructor(
    private setChainCommitment,
    private getChainCommitment,
    private db: HoprDB,
    private channelId: Hash // used in db key
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
    this.db.setCurrentCommitment(
      this.channelId,
      await this.findPreImage(await this.db.getCurrentCommitment(this.channelId))
    )
  }

  private async findPreImage(hash: Hash): Promise<Hash> {
    // TODO refactor after we move primitives
    let result = await recoverIteratedHash(
      hash.serialize(),
      hashFunction,
      async (x) => await this.searchDBFor(x),
      TOTAL_ITERATIONS,
      DB_ITERATION_BLOCK_SIZE
    )
    if (result == undefined) {
      throw Error(`Could not find preImage.`)
    }
    return new Hash(result.preImage)
  }

  private async initialize(): Promise<void> {
    if (this.initialized) return
    const dbContains = await this.hasDBSecret()
    const chainCommitment = await this.getChainCommitment()
    if (chainCommitment && dbContains) {
      try {
        await this.findPreImage(chainCommitment) // throws if not found
        await this.db.getCurrentCommitment(this.channelId) // Find out if we have one
        this.initialized = true
        return
      } catch (_e) {
        log(`Secret is found but failed to find preimage, reinitializing..`)
      }
    }
    log(`reinitializing (db: ${dbContains}, chain: ${chainCommitment}})`)
    await this.createCommitmentChain()
    this.initialized = true
  }

  private async createCommitmentChain(): Promise<void> {
    const seed = new Hash(randomBytes(Hash.SIZE)) // TODO seed off privKey + channel
    const result = await iterateHash(seed.serialize(), hashFunction, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE)
    await this.db.storeHashIntermediaries(this.channelId, result.intermediates)
    const current = new Hash(result.hash)
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
