import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { typedClass } from 'src/tsc/utils'
import { BYTES32 } from './solidity'

@typedClass<TypeConstructors['AccountId']>()
class AccountId extends BYTES32 {}

export default AccountId
