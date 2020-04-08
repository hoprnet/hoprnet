import type { Types } from '@hoprnet/hopr-core-connector-interface';
import { UINT256 } from './solidity';
declare class Balance extends UINT256 implements Types.Balance {
    static get SYMBOL(): string;
    static get DECIMALS(): number;
}
export default Balance;
