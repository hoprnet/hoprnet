import Address from './accountId'

declare interface PublicStatic {
  SIZE: number

  new (public: Uint8Array, ...props: any[]): Public
}

declare interface Public extends Uint8Array {
  toAddress(): Promise<Address>
}

declare var Public: PublicStatic

export default Public
