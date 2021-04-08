import { toU8a, serializeToU8a, Intermediate} from '@hoprnet/hopr-utils'
import { Hash } from './types'
import type { LevelUp } from 'levelup'

const encoder = new TextEncoder()
const PREFIX = encoder.encode('payments-')
const SEPERATOR = encoder.encode('-')

const ITERATION_WIDTH = 4 // bytes

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

function onChainSecretIntermediaryKey(iteration: number): Uint8Array {
  const onChainSecretIntermediary = encoder.encode('onChainSecretIntermediary-')
  return serializeToU8a([
    [PREFIX, PREFIX.length],
    [onChainSecretIntermediary, onChainSecretIntermediary.length],
    [SEPERATOR, SEPERATOR.length],
    [toU8a(iteration, ITERATION_WIDTH), ITERATION_WIDTH]
  ])
}

export async function getOnChainSecret(db: LevelUp): Promise<Hash> {
  return new Hash(await getFromDB(db, onChainSecretIntermediaryKey(0)))
}

export async function getOnChainSecretIntermediary(db: LevelUp, index: number): Promise<Uint8Array>{
  return getFromDB(db, onChainSecretIntermediaryKey(index))
}

export async function storeHashIntermediaries(db: LevelUp, intermediates: Intermediate[]): Promise<void> {
  let dbBatch = db.batch()
  for (const intermediate of intermediates) {
    dbBatch = dbBatch.put(
      Buffer.from(onChainSecretIntermediaryKey(intermediate.iteration)),
      Buffer.from(intermediate.preImage)
    )
  }
  await dbBatch.write()
}


