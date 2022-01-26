import { GlobalState } from '../../abstractCommand'
import { checkPeerIdInput, styleValue } from '../../utils'

export const setAlias = ({
  peerId,
  alias,
  state,
  log
}: {
  peerId: string
  alias: string
  state: GlobalState
  log?: (string) => void
}) => {
  try {
    let validPeerId = checkPeerIdInput(peerId)
    state.aliases.set(alias, validPeerId)

    log(`Set alias '${styleValue(alias, 'highlight')}' to '${styleValue(validPeerId.toB58String(), 'peerId')}'.`)
  } catch (error) {
    log(styleValue(error.message, 'failure'))
    return new Error('invalidPeerId')
  }
}

export const getAlias = ({ state, peerId }: { state: GlobalState; peerId: string }) => {
  const aliases = Array.from(state.aliases.entries())
    .filter(([_, peerIdInMap]) => peerIdInMap.toB58String() === peerId)
    .map(([alias, _]) => alias)

  return aliases.length > 0 ? aliases : new Error('aliasNotFound')
}

// unused yet
export const getPeerIdByAlias = ({ state, alias }: { state: GlobalState; alias: string }) => {
  return state.aliases.get(alias) || new Error('')
}
