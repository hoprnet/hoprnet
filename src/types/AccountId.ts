import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { typedClass } from '../tsc/utils'
import { BYTES32 } from './solidity'
import Web3 from 'web3'

@typedClass<TypeConstructors['AccountId']>()
class AccountId extends BYTES32 {
  toHex() {
    return Web3.utils.toChecksumAddress(super.toHex())
  }
}

export default AccountId
