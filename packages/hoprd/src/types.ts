import type PeerId from 'peer-id'

export enum SettingKey {
  STRATEGY = 'strategy',
  INCLUDE_RECIPIENT = 'includeRecipient'
}

/**
 * HOPRd specific state used by the daemon.
 */
export type State = {
  aliases: Map<string, PeerId>
  settings: {
    [SettingKey.STRATEGY]: 'passive' | 'promiscuous'
    [SettingKey.INCLUDE_RECIPIENT]: boolean
  }
}

export interface StateOps {
  setState: (newState: State) => void
  getState: () => State
}
