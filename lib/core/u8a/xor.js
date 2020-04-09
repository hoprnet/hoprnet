"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * Apply an XOR on a list of arrays.
 *
 * @param inPlace if `true` overwrite first Array with result
 * @param list arrays to XOR
 */
function u8aXOR(inPlace = false, ...list) {
    if (!list.slice(1).every(array => array.length == list[0].length)) {
        throw Error(`Uint8Array must not have different sizes`);
    }
    const result = inPlace ? list[0] : new Uint8Array(list[0].length);
    if (list.length == 2) {
        for (let index = 0; index < list[0].length; index++) {
            result[index] = list[0][index] ^ list[1][index];
        }
    }
    else {
        for (let index = 0; index < list[0].length; index++) {
            result[index] = list.reduce((acc, array) => acc ^ array[index], 0);
        }
    }
    return result;
}
exports.u8aXOR = u8aXOR;
