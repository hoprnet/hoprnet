import { TypeClasses } from './types'

type Utils = {
  isPartyA(self: TypeClasses.AccountId, counterparty: TypeClasses.AccountId): boolean
  getId(self: TypeClasses.AccountId, counterparty: TypeClasses.AccountId, ...props: any[]): Promise<TypeClasses.Hash>
  pubKeyToAccountId(pubkey: Uint8Array, ...args: any[]): Promise<TypeClasses.AccountId>
  hash(msg: Uint8Array): Promise<TypeClasses.Hash>
  sign(msg: Uint8Array, privKey: Uint8Array, pubKey: Uint8Array): Promise<{
    signature: Uint8Array,
    recovery: number
  }>
  verify(msg: Uint8Array, signature: Uint8Array, pubkey: Uint8Array): Promise<boolean>
}

export default Utils
