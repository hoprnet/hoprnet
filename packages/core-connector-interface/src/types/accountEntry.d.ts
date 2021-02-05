import type BN from 'bn.js'

declare interface AccountEntryStatic {
  readonly SIZE: number
}

declare interface AccountEntry {
  blockNumber: BN
  transactionIndex: BN
  logIndex: BN
  hashedSecret: Uint8Array
  counter: BN
}

declare var AccountEntry: AccountEntryStatic

export default AccountEntry
