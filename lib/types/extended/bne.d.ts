import BN from 'bn.js';
declare class BNE extends BN {
    toU8a(length?: number): Uint8Array;
}
export default BNE;
