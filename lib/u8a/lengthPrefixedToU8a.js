"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const constants_1 = require("./constants");
const u8aToNumber_1 = require("./u8aToNumber");
/**
 * Decodes a length-prefixed array and returns the encoded data.
 *
 * @param arg array to decode
 * @param additionalPadding additional padding to remove
 * @param targetLength optional target length
 */
function lengthPrefixedToU8a(arg, additionalPadding, targetLength) {
    if (targetLength != null && arg.length < targetLength) {
        throw Error(`Expected a ${Uint8Array.name} of at least lenght ${targetLength}`);
    }
    else if (arg.length < constants_1.LENGTH_PREFIX_LENGTH || (additionalPadding != null && arg.length < constants_1.LENGTH_PREFIX_LENGTH + additionalPadding.length)) {
        throw Error(`Expected a ${Uint8Array.name} of at least length ${additionalPadding != null ? constants_1.LENGTH_PREFIX_LENGTH + additionalPadding.length : constants_1.LENGTH_PREFIX_LENGTH} but got ${arg.length}.`);
    }
    let arrLength = u8aToNumber_1.u8aToNumber(arg.subarray(0, constants_1.LENGTH_PREFIX_LENGTH));
    if (!Number.isInteger(arrLength)) {
        throw Error(`Invalid encoded length.`);
    }
    if (targetLength == null &&
        (additionalPadding != null ? arrLength + additionalPadding.length + constants_1.LENGTH_PREFIX_LENGTH != arg.length : arrLength + constants_1.LENGTH_PREFIX_LENGTH != arg.length)) {
        throw Error(`Invalid array length. Expected a ${Uint8Array.name} of at least length ${additionalPadding != null ? constants_1.LENGTH_PREFIX_LENGTH + additionalPadding.length + arrLength : constants_1.LENGTH_PREFIX_LENGTH + arrLength} but got ${arg.length}.`);
    }
    if (additionalPadding != null &&
        arg
            .subarray(constants_1.LENGTH_PREFIX_LENGTH, constants_1.LENGTH_PREFIX_LENGTH + additionalPadding.length)
            .some((value, index) => value != additionalPadding[index])) {
        throw Error(`Array does not contain correct additional padding.`);
    }
    if (additionalPadding != null) {
        return arg.subarray(constants_1.LENGTH_PREFIX_LENGTH + additionalPadding.length, constants_1.LENGTH_PREFIX_LENGTH + additionalPadding.length + arrLength);
    }
    else {
        return arg.subarray(constants_1.LENGTH_PREFIX_LENGTH, constants_1.LENGTH_PREFIX_LENGTH + arrLength);
    }
}
exports.lengthPrefixedToU8a = lengthPrefixedToU8a;
