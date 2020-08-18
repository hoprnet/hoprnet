import AccountId from './accountId'

declare namespace Public {
  const SIZE: number
}

declare interface Public extends Uint8Array {
  new (public: Uint8Array, ...props: any[]): Public

  toAccountId(): Promise<AccountId>
}

export default Public
