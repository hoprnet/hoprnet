import type { Types } from '@hoprnet/hopr-core-connector-interface';
import { UINT256 } from './solidity';
declare class NativeBalance extends UINT256 implements Types.NativeBalance {
    static get SYMBOL(): string;
    static get DECIMALS(): number;
}
export default NativeBalance;
