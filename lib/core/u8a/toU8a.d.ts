/**
 * Converts a number to a Uint8Array and optionally adds some padding to match
 * the desired size.
 * @param arg to convert to Uint8Array
 * @param length desired length of the Uint8Array
 */
export declare function toU8a(arg: number, length?: number): Uint8Array;
/**
 * Converts a string to a Uint8Array and optionally adds some padding to match
 * the desired size.
 * @notice Throws an error in case a length was provided and the result does not fit.
 * @param str string to convert
 * @param length desired length of the Uint8Array
 */
export declare function stringToU8a(str: string, length?: number): Uint8Array;
