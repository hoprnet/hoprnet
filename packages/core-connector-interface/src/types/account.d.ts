import type BN from 'bn.js'
import type Address from './accountId'
import type Public from './public'
import type Hash from './hash'

declare interface AccountStatic {
  readonly SIZE: number
  new (address: Address, publicKey?: Public, secret?: Hash, counter?: BN): Account
}

declare interface Account {
  address: Address
  publicKey?: Public
  secret?: Hash
  counter?: BN
  isInitialized(): boolean
}

declare var Account: AccountStatic

export default Account
