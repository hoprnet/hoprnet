declare type MemoryPage = {
    page: ArrayBuffer;
    offset: number;
};
/**
 * Writes to the provided mempage the data on a given list of u8a on a given offset
 *
 * @export
 * @param {MemoryPage} { page: ArrayBuffer, offset: number }
 * @param {(...(Uint8Array | undefined)[])} list
 * @returns {Uint8Array}
 */
export declare function u8aAllocate({ page, offset }: MemoryPage, ...list: (Uint8Array | undefined)[]): Uint8Array;
export {};
