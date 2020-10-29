import type {Types} from '@hoprnet/hopr-core-connector-interface'
import {UINT256} from './solidity'

class Moment extends UINT256 implements Types.Moment {}

export default Moment
