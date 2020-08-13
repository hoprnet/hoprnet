import { BYTES32 } from './solidity';
import { Types } from '@hoprnet/hopr-core-connector-interface';
import AccountId from './accountId';
declare class Public extends BYTES32 implements Types.Public {
    get NAME(): string;
    toAccountId(): Promise<AccountId>;
    static get SIZE(): number;
}
export default Public;
