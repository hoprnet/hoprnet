import type { Types } from '@hoprnet/hopr-core-connector-interface';
import { BYTES32 } from './solidity';
declare class AccountId extends BYTES32 implements Types.AccountId {
    static get SIZE(): number;
    get NAME(): string;
    toHex(): string;
}
export default AccountId;
