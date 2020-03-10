import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { typedClass } from '../tsc/utils'
import { BYTES32 } from './solidity'

@typedClass<TypeConstructors['Hash']>()
class Hash extends BYTES32 {}

export default Hash
