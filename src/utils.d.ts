import { AccountId, Hash } from './types'

declare function isPartyA(self: AccountId, counterparty: AccountId): boolean

declare function getId(self: AccountId, counterparty: AccountId, api?: any): Promise<Hash>

declare function pubKeytToAccountId(pubkey: Uint8Array, ...args: any[]): Promise<AccountId>

declare function hash(msg: Uint8Array): Uint8Array

export default interface Utils {
  isPartyA: typeof isPartyA
  getId: typeof getId
  pubKeyToAccountId: typeof pubKeytToAccountId
  hash: typeof hash
}
