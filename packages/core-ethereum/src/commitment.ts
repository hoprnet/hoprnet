// See the specification for a full description of what this is for, but
// essentially we want to post a 'commitment' to each ticket, that allows
// later verification by giving a 'preimage' of that commitment.
//
// We need to persist this string of commitments in the database, and support
// syncing back and forth with those that have been persisted on chain.
import {iterateHash, recoverIteratedHash, HoprDB, Hash, ChannelEntry, u8aConcat} from '@hoprnet/hopr-utils'
import { debug } from '@hoprnet/hopr-utils'
import {deriveCommitmentSeed} from "@hoprnet/hopr-utils/lib/crypto/commitment/keyDerivation";

const log = debug('hopr-core-ethereum:commitment')

export const DB_ITERATION_BLOCK_SIZE = 10000
export const TOTAL_ITERATIONS = 100000

function hashFunction(msg: Uint8Array): Uint8Array {
  return Hash.create(msg).serialize().slice(0, Hash.SIZE)
}

function searchDBFor(db: HoprDB, channelId: Hash, iteration: number): Promise<Uint8Array | undefined> {
  return db.getCommitment(channelId, iteration)
}

export async function findCommitmentPreImage(db: HoprDB, channelId: Hash): Promise<Hash> {
  let currentCommitment = await db.getCurrentCommitment(channelId)
  let result = await recoverIteratedHash(
    currentCommitment.serialize(),
    hashFunction,
    (i: number) => searchDBFor(db, channelId, i),
    TOTAL_ITERATIONS,
    DB_ITERATION_BLOCK_SIZE
  )
  if (result == undefined) {
    throw Error(`Could not find preImage. Searching for ${currentCommitment.toHex()}`)
  }
  return new Hash(Uint8Array.from(result.preImage))
}

export async function bumpCommitment(db: HoprDB, channelId: Hash) {
  await db.setCurrentCommitment(channelId, await findCommitmentPreImage(db, channelId))
}

type GetCommitment = () => Promise<Hash>
type SetCommitment = (commitment: Hash) => Promise<string>

async function createCommitmentChain(db: HoprDB, channelId: Hash, channelMasterKey: Uint8Array, setChainCommitment: SetCommitment): Promise<void> {
  const seed = new Hash(channelMasterKey)
  const { intermediates, hash } = await iterateHash(
    seed.serialize(),
    hashFunction,
    TOTAL_ITERATIONS,
    DB_ITERATION_BLOCK_SIZE
  )
  await db.storeHashIntermediaries(channelId, intermediates)
  const current = new Hash(Uint8Array.from(hash))
  await Promise.all([db.setCurrentCommitment(channelId, current), setChainCommitment(current)])
  log('commitment chain initialized')
}

export async function initializeCommitment(
  db: HoprDB,
  privateKey: Uint8Array,
  channelEntry: ChannelEntry,
  getChainCommitment: GetCommitment,
  setChainCommitment: SetCommitment
) {
  const channelId = channelEntry.getId()
  const dbContainsAlready = (await db.getCommitment(channelId, 0)) != undefined
  const chainCommitment = await getChainCommitment()

  if (chainCommitment && dbContainsAlready) {
    try {
      await findCommitmentPreImage(db, channelId) // throws if not found
      return
    } catch (e) {
      log(`Secret is found but failed to find preimage, reinitializing.. ${e.message}`)
    }
  }
  log(`reinitializing (db: ${dbContainsAlready}, chain: ${chainCommitment}})`)

  // Information identifying this channel
  // TODO: Add contract address, chain id, ...etc.
  const channelInfo = u8aConcat(channelEntry.channelEpoch.serialize(), channelId.serialize())
  await createCommitmentChain(db, channelId, deriveCommitmentSeed(privateKey, channelInfo), setChainCommitment)
}
