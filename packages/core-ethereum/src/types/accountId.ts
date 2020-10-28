import Web3 from 'web3'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { ADDRESS_LENGTH } from '../constants'
import { Uint8ArrayE } from './extended'

class AccountId extends Uint8ArrayE implements Types.AccountId {
  slice(begin = 0, end = AccountId.SIZE): Uint8Array {
    return this.subarray(begin, end)
  }

  subarray(begin = 0, end = AccountId.SIZE): Uint8Array {
    return new Uint8Array(this.buffer, begin + this.byteOffset, end - begin)
  }

  static get SIZE(): number {
    return ADDRESS_LENGTH
  }

  get NAME(): string {
    return 'AccountId'
  }

  toHex(): string {
    return Web3.utils.toChecksumAddress(super.toHex())
  }
}

export default AccountId
