import { AccountId, Hash } from './types'

export function isPartyA(self: AccountId, counterparty: AccountId): boolean

export function getId(self: AccountId, counterparty: AccountId, api?: any): Promise<Hash>

export function pubKeytToAccountId(pubkey: Uint8Array, ...args: any[]): Promise<AccountId>

export function hash(msg: Uint8Array): Uint8Array
