import type { Types } from '@hoprnet/hopr-core-connector-interface'
import { BYTES32 } from './solidity'

class Hash extends BYTES32 implements Types.Hash {}

export default Hash
