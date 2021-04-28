import { Hash } from './types'
import { iterateHash, recoverIteratedHash, u8aConcat, Intermediate } from '@hoprnet/hopr-utils'
import type { LevelUp } from 'levelup'
import { randomBytes } from 'crypto'
import Debug from 'debug'
import { getFromDB } from './utils'

const log = Debug('hopr-core-ethereum:commitment')

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000

async function hashFunction(msg: Uint8Array): Promise<Uint8Array> {
  return Hash.create(msg).serialize().slice(0, Hash.SIZE)
}

function keyFor(channelId: Hash, iteration: number): Uint8Array {
  const prefix = new TextEncoder().encode('commitment:')
  return u8aConcat(prefix, channelId.serialize(), Uint8Array.of(iteration))
}

export async function storeHashIntermediaries(
  db: LevelUp,
  channelId: Hash,
  intermediates: Intermediate[]
): Promise<void> {
  let dbBatch = db.batch()
  for (const intermediate of intermediates) {
    dbBatch = dbBatch.put(Buffer.from(keyFor(channelId, intermediate.iteration)), Buffer.from(intermediate.preImage))
  }
  await dbBatch.write()
}

export class Commitment {
  private initialized: boolean = false
  private current: Hash

  constructor(
    private setChainCommitment,
    private getChainCommitment,
    private db: LevelUp,
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
        log(`Secret is found but failed to find preimage, reinitializing..`)
      }
    }
    log(`reinitializing (db: ${dbContains}, chain: ${chainCommitment}})`)
    this.current = await this.createCommitmentChain()
    await this.setChainCommitment(this.current)
    this.initialized = true
  }

  // returns last hash in chain
  private async createCommitmentChain(): Promise<Hash> {
    const seed = new Hash(randomBytes(Hash.SIZE)) // TODO seed off privKey + channel
    const result = await iterateHash(seed.serialize(), hashFunction, TOTAL_ITERATIONS, DB_ITERATION_BLOCK_SIZE)
    await storeHashIntermediaries(this.db, this.channelId, result.intermediates)
    return new Hash(result.hash)
  }

  private async hasDBSecret(): Promise<boolean> {
    return (await getFromDB<Uint8Array>(this.db, keyFor(this.channelId, 0))) != undefined
  }

  private async searchDBFor(iteration: number): Promise<Uint8Array | undefined> {
    return await getFromDB<Uint8Array>(this.db, keyFor(this.channelId, iteration))
  }
}
