import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { BYTES32 } from './solidity'
import Web3 from 'web3'

class AccountId extends BYTES32 implements Types.AccountId {
  toHex() {
    return Web3.utils.toChecksumAddress(super.toHex())
  }
}

export default AccountId
