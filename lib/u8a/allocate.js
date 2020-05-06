"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * Writes to the provided mempage the data on a given list of u8a on a given offset
 *
 * @export
 * @param {MemoryPage} { page: ArrayBuffer, offset: number }
 * @param {(...(Uint8Array | undefined)[])} list
 * @returns {Uint8Array}
 */
function u8aAllocate({ page, offset }, ...list) {
    let totalLength = 0;
    const listLength = list.length;
    for (let i = 0; i < listLength; i++) {
        if (list[i] !== undefined) {
            // @ts-ignore
            totalLength += list[i].length;
        }
    }
    const pageLength = page.byteLength;
    if (listLength > pageLength) {
        throw new Error('Error: The length of the provided arrays is bigger than the given page');
    }
    if (offset + totalLength > pageLength) {
        throw new Error('Error: The offset given is not enough for allocating the given arrays');
    }
    const result = new Uint8Array(page, offset, totalLength);
    let pageOffset = 0;
    for (let i = 0; i < listLength; i++) {
        if (list[i] !== undefined) {
            // @ts-ignore
            result.set(list[i], pageOffset);
            // @ts-ignore
            pageOffset += list[i].length;
        }
    }
    return result;
}
exports.u8aAllocate = u8aAllocate;
