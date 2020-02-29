import TypeConstructors from '@hoprnet/hopr-core-connector-interface/src/types'
import { typedClass } from 'src/tsc/utils'
import { UINT256 } from './solidity'

@typedClass<TypeConstructors['Balance']>()
class Balance extends UINT256 {
  static get SYMBOL(): string {
    return `HOPR`
  }

  static get DECIMALS(): number {
    return 18
  }
}

export default Balance
