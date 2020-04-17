"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const crypto_1 = require("crypto");
/**
 * @param start
 * @param end
 * @returns random number between @param start and @param end
 */
function randomInteger(start, end) {
    if (start < 0 || (end != null && end < 0)) {
        throw Error(`'start' and 'end' must be positive.`);
    }
    if (end != null) {
        if (start >= end) {
            throw Error(`Invalid interval. 'end' must be strictly greater than 'start'.`);
        }
        if (start + 1 == end) {
            return start;
        }
    }
    // Projects interval from [start, end) to [0, end - start)
    let interval = end == null ? start : end - start;
    if (interval >= Math.pow(2, 32)) {
        throw Error(`Not implemented`);
    }
    const byteAmount = 32 - Math.clz32(interval - 1);
    let bytes = crypto_1.randomBytes(Math.max(byteAmount / 8, 1));
    let bitCounter = 0;
    let byteCounter = 0;
    function nextBit() {
        let result = bytes[byteCounter] % 2;
        bytes[byteCounter] = bytes[byteCounter] >> 1;
        if (++bitCounter == 8) {
            bitCounter = 0;
            byteCounter++;
        }
        return result;
    }
    let result = 0;
    for (let i = 0; i < byteAmount; i++) {
        if ((result | (1 << i)) < interval) {
            if (nextBit() == 1) {
                result |= 1 << i;
            }
        }
    }
    return end == null ? result : start + result;
}
exports.randomInteger = randomInteger;
