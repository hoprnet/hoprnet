import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { typedClass } from 'src/tsc/utils'
import { UINT256 } from './solidity'

@typedClass<TypeConstructors['Moment']>()
class Moment extends UINT256 {}

export default Moment
