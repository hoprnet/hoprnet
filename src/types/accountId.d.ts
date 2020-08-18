declare interface AccountIdStatic {
  readonly SIZE: number

  new (accountId: Uint8Array, ...props: any[]): AccountId
}

declare interface AccountId extends Uint8Array {}

declare var AccountId: AccountIdStatic

export default AccountId
