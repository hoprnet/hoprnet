import chalk from 'chalk'

import PeerId from 'peer-id'
import Multihash from 'multihashes'
import bs58 from 'bs58'

import { addPubKey } from '@hoprnet/hopr-core/lib/src/utils'

/**
 * Takes the string representation of a peerId and checks whether it is a valid
 * peerId, i. e. it is a valid base58 encoding.
 * It then generates a PeerId instance and returns it.
 *
 * @param query query that contains the peerId
 */
export async function checkPeerIdInput(query: string): Promise<PeerId> {
  let peerId: PeerId

  try {
    // Throws an error if the Id is invalid
    Multihash.decode(bs58.decode(query))

    peerId = await addPubKey(PeerId.createFromB58String(query))
  } catch (err) {
    throw Error(chalk.red(`Invalid peerId. ${err.message}`))
  }

  return peerId
}
