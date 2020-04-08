"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const toU8a_1 = require("./toU8a");
describe('test number to u8a', function () {
    it('should return a u8a', function () {
        assert_1.default.deepEqual(toU8a_1.toU8a(0), new Uint8Array([0x00]));
        assert_1.default.deepEqual(toU8a_1.toU8a(1), new Uint8Array([0x01]));
        assert_1.default.deepEqual(toU8a_1.toU8a(1234), new Uint8Array([0x04, 0xd2]));
        assert_1.default.deepEqual(toU8a_1.toU8a(12345), new Uint8Array([0x30, 0x39]));
        assert_1.default.throws(() => toU8a_1.toU8a(-1));
        assert_1.default.throws(() => toU8a_1.toU8a(NaN));
        assert_1.default.throws(() => toU8a_1.toU8a(Infinity));
    });
    it('should return a fixed-size u8a', function () {
        assert_1.default.deepEqual(toU8a_1.toU8a(0, 1), new Uint8Array([0x00]));
        assert_1.default.deepEqual(toU8a_1.toU8a(1, 1), new Uint8Array([0x01]));
        assert_1.default.deepEqual(toU8a_1.toU8a(1234, 2), new Uint8Array([0x04, 0xd2]));
        assert_1.default.deepEqual(toU8a_1.toU8a(12345, 2), new Uint8Array([0x30, 0x39]));
        assert_1.default.throws(() => toU8a_1.toU8a(-1, 123));
        assert_1.default.throws(() => toU8a_1.toU8a(NaN, 1234));
        assert_1.default.throws(() => toU8a_1.toU8a(Infinity, 12345));
        assert_1.default.throws(() => toU8a_1.toU8a(12345, 1));
        assert_1.default.deepEqual(toU8a_1.toU8a(1, 2), new Uint8Array([0x00, 0x01]));
        assert_1.default.deepEqual(toU8a_1.toU8a(1, 3), new Uint8Array([0x00, 0x00, 0x01]));
    });
    it('should return a u8a', function () {
        assert_1.default.deepEqual(toU8a_1.stringToU8a('0x123'), new Uint8Array([0x01, 0x23]));
        assert_1.default.deepEqual(toU8a_1.stringToU8a('123'), new Uint8Array([0x01, 0x23]));
        assert_1.default.deepEqual(toU8a_1.stringToU8a('0x23'), new Uint8Array([0x23]));
        assert_1.default.deepEqual(toU8a_1.stringToU8a('23'), new Uint8Array([0x23]));
        assert_1.default.throws(() => toU8a_1.stringToU8a('g'), 'Should throw on non-Hex Strings');
        assert_1.default.throws(() => toU8a_1.stringToU8a('0x0g'), 'Should throw on non-Hex Strings');
        assert_1.default.throws(() => toU8a_1.stringToU8a('0x000g'), 'Should throw on non-Hex Strings');
    });
});
//# sourceMappingURL=toU8a.spec.js.map