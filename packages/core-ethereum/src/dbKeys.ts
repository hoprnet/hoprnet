import { toU8a, serializeToU8a, Intermediate} from '@hoprnet/hopr-utils'
import { Hash, PublicKey } from './types'
import type { LevelUp } from 'levelup'

const encoder = new TextEncoder()
const PREFIX = encoder.encode('payments-')
const SEPERATOR = encoder.encode('-')
const ticketSubPrefix = encoder.encode('tickets-')
const acknowledgedSubPrefix = encoder.encode('acknowledged-')

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

/**
 * Returns the db-key under which the tickets are saved in the database.
 */
export function AcknowledgedTicket(counterPartyPubKey: PublicKey, challenge: Hash): Uint8Array {
  return serializeToU8a([
    [ticketSubPrefix, ticketSubPrefix.length],
    [acknowledgedSubPrefix, acknowledgedSubPrefix.length],
    [counterPartyPubKey.serialize(), PublicKey.SIZE],
    [SEPERATOR, SEPERATOR.length],
    [challenge.serialize(), Hash.SIZE]
  ])
}

/**
 * Reconstructs counterPartyPubKey and the specified challenge from a AcknowledgedTicket db-key.
 * @param arr a AcknowledgedTicket db-key
 * @param props additional arguments
 */
export function AcknowledgedTicketParse(arr: Uint8Array): [PublicKey, Hash] {
  const counterPartyPubKeyStart = ticketSubPrefix.length + acknowledgedSubPrefix.length
  const counterPartyPubKeyEnd = counterPartyPubKeyStart + PublicKey.SIZE
  const challengeStart = counterPartyPubKeyEnd + SEPERATOR.length
  const challengeEnd = challengeStart + Hash.SIZE

  return [
    new PublicKey(arr.slice(counterPartyPubKeyStart, counterPartyPubKeyEnd)),
    new Hash(arr.slice(challengeStart, challengeEnd))
  ]
}
