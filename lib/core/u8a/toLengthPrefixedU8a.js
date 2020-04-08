"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const toU8a_1 = require("./toU8a");
const concat_1 = require("./concat");
const constants_1 = require("../constants");
const _1 = require(".");
/**
 * Adds a length-prefix to a Uint8Array
 * @param arg data to add padding
 * @param additionalPadding optional additional padding that is inserted between length and data
 * @param length optional target length
 *
 */
function toLengthPrefixedU8a(arg, additionalPadding, length) {
    if (additionalPadding != null) {
        if (length != null && arg.length + _1.LENGTH_PREFIX_LENGTH + additionalPadding.length > length) {
            throw Error(`Cannot create length-prefixed ${Uint8Array.name} because encoded ${Uint8Array.name} would be ${length -
                arg.length +
                _1.LENGTH_PREFIX_LENGTH +
                additionalPadding.length} bytes greater than packet size of ${constants_1.PACKET_SIZE} bytes.`);
        }
        if (length != null) {
            return concat_1.u8aConcat(toU8a_1.toU8a(arg.length, _1.LENGTH_PREFIX_LENGTH), additionalPadding, arg, new Uint8Array(length - _1.LENGTH_PREFIX_LENGTH - additionalPadding.length - arg.length));
        }
        else {
            return concat_1.u8aConcat(toU8a_1.toU8a(arg.length, _1.LENGTH_PREFIX_LENGTH), additionalPadding, arg);
        }
    }
    else {
        if (length != null && arg.length + _1.LENGTH_PREFIX_LENGTH > length) {
            throw Error(`Cannot create length-prefixed ${Uint8Array.name} because encoded ${Uint8Array.name} would be ${length -
                arg.length +
                _1.LENGTH_PREFIX_LENGTH} bytes greater than packet size of ${constants_1.PACKET_SIZE} bytes.`);
        }
        if (length != null) {
            return concat_1.u8aConcat(toU8a_1.toU8a(arg.length, _1.LENGTH_PREFIX_LENGTH), arg, new Uint8Array(length - _1.LENGTH_PREFIX_LENGTH - arg.length));
        }
        else {
            return concat_1.u8aConcat(toU8a_1.toU8a(arg.length, _1.LENGTH_PREFIX_LENGTH), arg);
        }
    }
}
exports.toLengthPrefixedU8a = toLengthPrefixedU8a;
//# sourceMappingURL=toLengthPrefixedU8a.js.map