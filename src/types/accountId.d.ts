declare namespace AccountId {
  const SIZE: number
}

declare interface AccountId extends Uint8Array {
  new (accountId: Uint8Array, ...props: any[]): AccountId
}

export default AccountId
