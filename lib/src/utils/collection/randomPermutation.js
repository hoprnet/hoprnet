"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const randomInteger_1 = require("../general/randomInteger");
/**
 * Return a random permutation of the given `array`
 * by using the (optimized) Fisher-Yates shuffling algorithm.
 *
 * @param array the array to permutate
 *
 * @example
 *
 * ```javascript
 * randomPermutation([1,2,3,4]);
 * // first run: [2,4,1,2]
 * // second run: [3,1,2,4]
 * // ...
 * ```
 */
function randomPermutation(array) {
    if (array.length <= 1) {
        return array;
    }
    let j;
    let tmp;
    for (let i = array.length - 1; i > 0; i--) {
        j = randomInteger_1.randomInteger(0, i + 1);
        tmp = array[i];
        array[i] = array[j];
        array[j] = tmp;
    }
    return array;
}
exports.randomPermutation = randomPermutation;
