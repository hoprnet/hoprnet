// See the specification for a full description of what this is for, but
// essentially we want to post a 'commitment' to each ticket, that allows
// later verification by giving a 'preimage' of that commitment.
//
// We need to persist this string of commitments in the database, and support
// syncing back and forth with those that have been persisted on chain.
import {
  ChannelEntry,
  debug,
  Hash,
  HoprDB,
  iterateHash,
  recoverIteratedHash,
  toU8a,
  u8aConcat
} from '@hoprnet/hopr-utils'
import { deriveCommitmentSeed } from '@hoprnet/hopr-utils/lib/crypto/commitment/keyDerivation'
import { toUtf8Bytes } from 'ethers/lib/utils'

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

async function createCommitmentChain(
  db: HoprDB,
  channelId: Hash,
  initialCommitmentSeed: Uint8Array,
  setChainCommitment: SetCommitment
): Promise<void> {
  const { intermediates, hash } = await iterateHash(
    initialCommitmentSeed,
    hashFunction,
    TOTAL_ITERATIONS,
    DB_ITERATION_BLOCK_SIZE
  )
  await db.storeHashIntermediaries(channelId, intermediates)
  const current = new Hash(Uint8Array.from(hash))
  await Promise.all([db.setCurrentCommitment(channelId, current), setChainCommitment(current)])
  log('commitment chain initialized')
}

/**
 * Simple class encapsulating channel information
 * used to generate the initial channel commitment.
 */
export class ChannelCommitmentInfo {
  constructor(
    public readonly channelEntry: ChannelEntry,
    public readonly chainId: number,
    public readonly contractAddress: string
  ) {}

  /**
   * Generate the initial commitment seed using this channel information and the given
   * private node key.
   * @param nodePrivateKey Local node private key.
   */
  public createInitialCommitmentSeed(nodePrivateKey: Uint8Array): Uint8Array {
    const channelSeedInfo = u8aConcat(
      this.channelEntry.channelEpoch.serialize(),
      toU8a(this.chainId, 4),
      this.channelEntry.getId().serialize(),
      toUtf8Bytes(this.contractAddress)
    )

    return deriveCommitmentSeed(nodePrivateKey, channelSeedInfo)
  }
}

export async function initializeCommitment(
  db: HoprDB,
  nodePrivateKey: Uint8Array,
  channelInfo: ChannelCommitmentInfo,
  getChainCommitment: GetCommitment,
  setChainCommitment: SetCommitment
) {
  const channelId = channelInfo.channelEntry.getId()
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
  await createCommitmentChain(
    db,
    channelId,
    channelInfo.createInitialCommitmentSeed(nodePrivateKey),
    setChainCommitment
  )
}
