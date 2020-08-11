import Web3 from 'web3'
import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { BYTES32 } from './solidity'
import { ADDRESS_LENGTH } from '../constants'

class AccountId extends BYTES32 implements Types.AccountId {
  static get SIZE() {
    return ADDRESS_LENGTH
  }

  get NAME() {
    return 'AccountId'
  }

  toHex() {
    return Web3.utils.toChecksumAddress(super.toHex())
  }
}

export default AccountId
