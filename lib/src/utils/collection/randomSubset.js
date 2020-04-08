"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const randomInteger_1 = require("../general/randomInteger");
const randomPermutation_1 = require("./randomPermutation");
/**
 * Picks @param subsetSize elements at random from @param array .
 * The order of the picked elements does not coincide with their
 * order in @param array
 *
 * @param array the array to pick the elements from
 * @param subsetSize the requested size of the subset
 * @param filter called with `(peerInfo)` and should return `true`
 * for every node that should be in the subset
 *
 * @returns array with at most @param subsetSize elements
 * that pass the test.
 *
 * @notice If less than @param subsetSize elements pass the test,
 * the result will contain less than @param subsetSize elements.
 */
function randomSubset(array, subsetSize, filter) {
    if (subsetSize < 0) {
        throw Error(`Invalid input arguments. Please provide a positive subset size. Got '${subsetSize}' instead.`);
    }
    if (subsetSize > array.length) {
        throw Error(`Invalid subset size. Subset size must not be greater than the array size.`);
    }
    if (subsetSize == 0) {
        return [];
    }
    if (subsetSize == array.length) {
        // Returns a random permutation of all elements that pass
        // the test.
        return randomPermutation_1.randomPermutation(filter != null ? array.filter(filter) : array);
    }
    if (subsetSize == 1) {
        let i = 0;
        let index = randomInteger_1.randomInteger(0, array.length);
        while (filter != null && !filter(array[index])) {
            if (i == array.length) {
                // There seems to be no element in the array
                // that passes the test.
                return [];
            }
            i++;
            index = (index + 1) % array.length;
        }
        return [array[index]];
    }
    let notChosen = new Set();
    let chosen = new Set();
    let found;
    let breakUp = false;
    let index;
    for (let i = 0; i < subsetSize && !breakUp; i++) {
        index = randomInteger_1.randomInteger(0, array.length);
        found = false;
        do {
            while (chosen.has(index) || notChosen.has(index)) {
                index = (index + 1) % array.length;
            }
            if (filter != null && !filter(array[index])) {
                notChosen.add(index);
                index = (index + 1) % array.length;
                found = false;
            }
            else {
                chosen.add(index);
                found = true;
            }
            if (notChosen.size + chosen.size == array.length) {
                breakUp = true;
                break;
            }
        } while (!found);
    }
    const result = [];
    for (let index of chosen) {
        result.push(array[index]);
    }
    return result;
}
exports.randomSubset = randomSubset;
