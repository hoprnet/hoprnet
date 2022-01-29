import { checkPeerIdInput } from '../../../commands/utils'
import { APIv2State } from '../../v2'

export const setAlias = ({ peerId, alias, state }: { peerId: string; alias: string; state: APIv2State }) => {
  try {
    let validPeerId = checkPeerIdInput(peerId)
    state.aliases.set(alias, validPeerId)
  } catch (error) {
    return new Error('invalidPeerId')
  }
}

export const getAlias = ({ state, peerId }: { state: APIv2State; peerId: string }) => {
  try {
    checkPeerIdInput(peerId)
  } catch (error) {
    return new Error('invalidPeerId')
  }
  const aliases = Array.from(state.aliases.entries())
    .filter(([_, peerIdInMap]) => peerIdInMap.toB58String() === peerId)
    .map(([alias, _]) => alias)

  return aliases.length > 0 ? aliases : new Error('aliasNotFound')
}
