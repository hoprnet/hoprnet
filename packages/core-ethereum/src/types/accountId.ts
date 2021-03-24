import Web3 from 'web3'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { ADDRESS_LENGTH } from '../constants'
import { u8aToHex, u8aEquals } from '@hoprnet/hopr-utils'

class AccountId implements Types.AccountId {
  constructor(private id: Uint8Array){}

  static get SIZE(): number {
    return ADDRESS_LENGTH
  }

  serialize() {
    return this.id
  }

  toHex(): string {
    return Web3.utils.toChecksumAddress(u8aToHex(this.id, false))
  }

  eq(b: AccountId) {
    return u8aEquals(this.id, b.serialize())
  }
}

export default AccountId
