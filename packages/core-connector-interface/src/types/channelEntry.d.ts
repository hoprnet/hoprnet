import type BN from 'bn.js'

declare interface ChannelEntryStatic {
  readonly SIZE: number
}

declare interface ChannelEntry {
  blockNumber: BN
  transactionIndex: BN
  logIndex: BN
  deposit: BN
  partyABalance: BN
  closureTime: BN
  stateCounter: BN
  closureByPartyA: boolean
  readonly status: 'UNINITIALISED' | 'FUNDED' | 'OPEN' | 'PENDING'
}

declare var ChannelEntry: ChannelEntryStatic

export default ChannelEntry
