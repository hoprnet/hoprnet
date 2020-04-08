"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const lengthPrefixedToU8a_1 = require("./lengthPrefixedToU8a");
const toLengthPrefixedU8a_1 = require("./toLengthPrefixedU8a");
describe('test length-prefixed to u8a', function () {
    it('should convert a length-prefixed u8a to u8a', function () {
        assert_1.default.deepEqual(lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 255])), new Uint8Array([255]));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 255, 1])));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1])));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0])));
        assert_1.default.deepEqual(lengthPrefixedToU8a_1.lengthPrefixedToU8a(toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array([1, 2, 3, 4]))), new Uint8Array([1, 2, 3, 4]));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([]), null, 1));
        assert_1.default.deepEqual(lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 0]), null, 6), new Uint8Array([1]));
    });
    it('should convert a length-prefixed u8a with additional padding to u8a', function () {
        assert_1.default.deepEqual(lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 255]), new Uint8Array([1])), new Uint8Array([255]));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 255]), new Uint8Array([2])));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 255]), new Uint8Array([2])));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 255, 1]), new Uint8Array([1])));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1]), new Uint8Array([1])));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0]), new Uint8Array([1])));
        assert_1.default.deepEqual(lengthPrefixedToU8a_1.lengthPrefixedToU8a(toLengthPrefixedU8a_1.toLengthPrefixedU8a(new Uint8Array([1, 2, 3, 4]), new Uint8Array([1])), new Uint8Array([1])), new Uint8Array([1, 2, 3, 4]));
        assert_1.default.throws(() => lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([]), new Uint8Array([1]), 2));
        assert_1.default.deepEqual(lengthPrefixedToU8a_1.lengthPrefixedToU8a(new Uint8Array([0, 0, 0, 1, 1, 1, 0]), null, 7), new Uint8Array([1])), new Uint8Array([1]);
    });
});
//# sourceMappingURL=lengthPrefixedToU8a.spec.js.map