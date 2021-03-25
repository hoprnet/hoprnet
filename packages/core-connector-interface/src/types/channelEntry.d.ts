import type BN from 'bn.js'
import type AccountId from './accountId'

declare interface ChannelEntryStatic {
  readonly SIZE: number
}

declare interface ChannelEntry {
  parties: [AccountId, AccountId]
  deposit: BN
  partyABalance: BN
  closureTime: BN
  stateCounter: BN
  closureByPartyA: boolean
  getStatus(): 'UNINITIALISED' | 'FUNDED' | 'OPEN' | 'PENDING'
  getIteration(): number
}

declare var ChannelEntry: ChannelEntryStatic

export default ChannelEntry
