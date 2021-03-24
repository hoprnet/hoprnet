declare interface AccountIdStatic {
  readonly SIZE: number
  new (accountId: Uint8Array): AccountId
}

declare interface AccountId {
  serialize(): Uint8Array
  eq(b: AccountId): boolean
  toHex(): string
}

declare var AccountId: AccountIdStatic

export default AccountId
