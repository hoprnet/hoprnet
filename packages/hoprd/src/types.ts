import type { PeerId } from '@libp2p/interface-peer-id'

export enum SettingKey {
//  STRATEGY = 'strategy',
  INCLUDE_RECIPIENT = 'includeRecipient',

//  MAX_AUTO_CHANNELS = 'maxAutoChannels',

//  AUTO_REDEEM_TICKETS = 'autoRedeemTickets'
}

/**
 * HOPRd specific state used by the daemon.
 */
export type State = {
  aliases: Map<string, PeerId>
  settings: {
    //[SettingKey.STRATEGY]: Strategy
    [SettingKey.INCLUDE_RECIPIENT]: boolean
    //[SettingKey.MAX_AUTO_CHANNELS]: number
    //[SettingKey.AUTO_REDEEM_TICKETS]: boolean
  }
}

export interface StateOps {
  setState: (newState: State) => void
  getState: () => State
}
