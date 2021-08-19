import { utils } from 'ethers'
import PeerId from 'peer-id'

/**
 * Verifies a given signature comes from a specific PeerId, based on the
 * signature generated and the PeerId id.
 *
 * @notice Currently we assume that the peerId was generated with a sec256k1
 * key, but no other tests had been done for additional keys (e.g. Curve25519)
 *
 * @param peerId the base58String representation of the PeerId
 * @param message the message signed by the given PeerId
 * @param signature the generated signature created by the PeerId
 */
export async function verifySignatureFromPeerId(peerId: string, message: string, signature: string): Promise<boolean> {
  const pId = await PeerId.createFromB58String(peerId)
  return await pId.pubKey.verify(new TextEncoder().encode(message), utils.arrayify(signature))
}
