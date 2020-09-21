import AccountId from './accountId'

declare interface PublicStatic {
  SIZE: number

  new (public: Uint8Array, ...props: any[]): Public
}

declare interface Public extends Uint8Array {
  toAccountId(): Promise<AccountId>
}

declare var Public: PublicStatic

export default Public
