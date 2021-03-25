declare interface AddressStatic {
  readonly SIZE: number
  new (accountId: Uint8Array): Address
  fromString(str: string): Address
}

declare interface Address {
  serialize(): Uint8Array
  eq(b: Address): boolean
  toHex(): string
}

declare var Address: AddressStatic

export default Address
