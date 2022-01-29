import { GlobalState } from '../../../commands/abstractCommand'
import { checkPeerIdInput } from '../../../commands/utils'

export const setAlias = ({ peerId, alias, state }: { peerId: string; alias: string; state: GlobalState }) => {
  try {
    let validPeerId = checkPeerIdInput(peerId)
    state.aliases.set(alias, validPeerId)
  } catch (error) {
    return new Error('invalidPeerId')
  }
}

export const getAlias = ({ state, peerId }: { state: GlobalState; peerId: string }) => {
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
