import PeerId from 'peer-id'
import { getAliases } from '../../fetch'

/**
 * Takes a string, and checks whether it's an alias or a valid peerId,
 * then it generates a PeerId instance and returns it.
 *
 * @param peerIdString query that contains the peerId
 * @returns a 'PeerId' instance
 */
export const checkPeerIdInput = async (peerIdString: string): Promise<PeerId> => {
  const aliases : string[] = await getAliases().then(res => Object.values(res))

  try {
    if (typeof aliases !== 'undefined' && aliases && aliases.includes(peerIdString)) {
      return PeerId.createFromB58String(peerIdString)
    }

    return PeerId.createFromB58String(peerIdString)
  } catch (err) {
    throw Error(`Invalid peerId. ${err.message}`)
  }
}
