import { TypeClasses } from './types'

type Utils = {
  isPartyA(self: TypeClasses.AccountId, counterparty: TypeClasses.AccountId): boolean
  getId(self: TypeClasses.AccountId, counterparty: TypeClasses.AccountId, ...props: any[]): Promise<TypeClasses.Hash>
  pubKeyToAccountId(pubkey: Uint8Array, ...args: any[]): Promise<TypeClasses.AccountId>
  hash(msg: Uint8Array): Promise<TypeClasses.Hash>
}

export default Utils
