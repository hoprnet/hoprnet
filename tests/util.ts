import PeerId from 'peer-id'
import { getIdentity } from './identities'

export async function peerIdForIdentity(identityName: string): Promise<PeerId> {
  return PeerId.createFromPrivKey(getIdentity(identityName))
}
