import { iterateHash, recoverIteratedHash, HoprDB, Hash } from '@hoprnet/hopr-utils'
import { randomBytes } from 'crypto'
import { Logger } from '@hoprnet/hopr-utils'

const log = Logger.getLogger('hopr-core-ethereum.commitment')

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000

async function hashFunction(msg: Uint8Array): Promise<Uint8Array> {
  return Hash.create(msg).serialize().slice(0, Hash.SIZE)
}

export class Commitment {
  private initialized: boolean = false
  private current: Hash

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
    return this.current
  }

  public async bumpCommitment() {
    if (!this.initialized) {
      await this.initialize()
    }
    this.current = await this.findPreImage(this.current)
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
        this.current = chainCommitment
        this.initialized = true
        return
      } catch (_e) {
        log.warn(`Secret is found but failed to find preimage, reinitializing..`)
      }
    }
    log.info(`Reinitializing (db: ${dbContains}, chain: ${chainCommitment}})`)
    this.current = await this.createCommitmentChain()
    await this.setChainCommitment(this.current)
    this.initialized = true
  }

  // returns last hash in chain
  private async createCommitmentChain(): Promise<Hash> {
    const seed = new Hash(randomBytes(Hash.SIZE)) // TODO seed off privKey + channel
    const result = await iterateHash(seed.serialize(), hashFunction, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE)
    await this.db.storeHashIntermediaries(this.channelId, result.intermediates)
    return new Hash(result.hash)
  }

  private async hasDBSecret(): Promise<boolean> {
    return (await this.db.getCommitment(this.channelId, 0)) != undefined
  }

  private async searchDBFor(iteration: number): Promise<Uint8Array | undefined> {
    return await this.db.getCommitment(this.channelId, iteration)
  }
}
