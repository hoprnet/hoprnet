/**
 * Decodes a length-prefixed array and returns the encoded data.
 *
 * @param arg array to decode
 * @param additionalPadding additional padding to remove
 * @param targetLength optional target length
 */
export declare function lengthPrefixedToU8a(arg: Uint8Array, additionalPadding?: Uint8Array, targetLength?: number): Uint8Array;
