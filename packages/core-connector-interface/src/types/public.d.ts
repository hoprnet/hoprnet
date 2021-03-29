import { Address } from '.' // TODO: cyclic

declare interface PublicStatic {
  SIZE: number
  new (public: Uint8Array, ...props: any[]): Public
  fromString(str: string): Public
}

declare interface Public extends Uint8Array {
  toAddress(): Promise<Address>
}

declare var Public: PublicStatic

export default Public
