declare class Uint8ArrayE extends Uint8Array {
    subarray(begin?: number, end?: number): Uint8Array;
    toU8a(): Uint8Array;
    toHex(): string;
    eq(b: Uint8Array): boolean;
}
export default Uint8ArrayE;
