import type PeerId from 'peer-id'

/**
 * HOPRd specific state used by the daemon.
 */
export type State = {
  aliases: Map<string, PeerId>
  settings: {
    strategy: 'passive' | 'promiscuous'
    includeRecipient: boolean
  }
}

export interface StateOps {
  setState: (newState: State) => void
  getState: () => State
}
