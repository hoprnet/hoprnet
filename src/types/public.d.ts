import AccountId from './accountId'

declare namespace Public {
  const SIZE: number
}

declare interface Public extends Uint8Array {
  toAccountId(): Promise<AccountId>
}

export default Public
