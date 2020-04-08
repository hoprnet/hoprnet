/**
 * Checks if the contents of the given Uint8Arrays are equal. Returns once at least
 * one different entry is found.
 * @param a first array
 * @param b second array
 * @param arrays additional arrays
 */
declare function u8aEquals(a: Uint8Array, b: Uint8Array, ...arrays: Uint8Array[]): boolean;
export { u8aEquals };
