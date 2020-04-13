"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const toLengthPrefixedU8a_1 = require("./toLengthPrefixedU8a");
const concat_1 = require("./concat");
describe('test u8a to length-prefixed u8a', function () {
    it('should return a length-prefixed u8a', function () {
        assert_1.default.deepEqual(toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array([1])), new Uint8Array([0, 0, 0, 1, 1]));
        assert_1.default.deepEqual(toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array(256)), concat_1.u8aConcat(new Uint8Array([0, 0, 1, 0]), new Uint8Array(256)));
        assert_1.default.throws(() => toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array([1]), null, 2));
        assert_1.default.deepEqual(toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array([1]), null, 6), new Uint8Array([0, 0, 0, 1, 1, 0]));
    });
    it('should return a length-prefixed u8a with additional padding', function () {
        assert_1.default.deepEqual(toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array([1]), new Uint8Array([1])), new Uint8Array([0, 0, 0, 1, 1, 1]));
        assert_1.default.deepEqual(toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array(256), new Uint8Array([1])), concat_1.u8aConcat(new Uint8Array([0, 0, 1, 0]), new Uint8Array([1]), new Uint8Array(256)));
        assert_1.default.throws(() => toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array([1]), new Uint8Array([1]), 5));
        assert_1.default.deepEqual(toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array([1]), new Uint8Array([1]), 7), new Uint8Array([0, 0, 0, 1, 1, 1, 0]));
    });
});
