import { BNE } from '../../types/extended';
declare class UINT256 extends BNE {
    toU8a(): Uint8Array;
    static get SIZE(): number;
}
export default UINT256;
