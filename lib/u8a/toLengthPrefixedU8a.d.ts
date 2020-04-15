/**
 * Adds a length-prefix to a Uint8Array
 * @param arg data to add padding
 * @param additionalPadding optional additional padding that is inserted between length and data
 * @param length optional target length
 *
 */
export declare function toLengthPrefixedU8a(arg: Uint8Array, additionalPadding?: Uint8Array, length?: number): Uint8Array;
