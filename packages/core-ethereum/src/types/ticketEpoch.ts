import type {Types} from '@hoprnet/hopr-core-connector-interface'
import {UINT256} from './solidity'

class TicketEpoch extends UINT256 implements Types.TicketEpoch {}

export default TicketEpoch
