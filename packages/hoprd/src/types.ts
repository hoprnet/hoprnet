import type { PeerId } from '@libp2p/interface-peer-id'
import { Strategy } from '@hoprnet/hopr-core'

export enum SettingKey {
  STRATEGY = 'strategy',
  INCLUDE_RECIPIENT = 'includeRecipient',

  MAX_AUTO_CHANNELS = 'maxAutoChannels'
}

/**
 * HOPRd specific state used by the daemon.
 */
export type State = {
  aliases: Map<string, PeerId>
  settings: {
    [SettingKey.STRATEGY]: Strategy
    [SettingKey.INCLUDE_RECIPIENT]: boolean
    [SettingKey.MAX_AUTO_CHANNELS]: number
  }
}

export interface StateOps {
  setState: (newState: State) => void
  getState: () => State
}
