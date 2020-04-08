"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const uint8ArrayE_1 = __importDefault(require("./uint8ArrayE"));
describe('test Uint8ArrayE', function () {
    const arr = new uint8ArrayE_1.default([1, 2, 3, 4, 5]);
    const hex = '0x0102030405';
    it('should return an equal Uint8Array', function () {
        const r = arr.toU8a();
        assert_1.default.deepEqual(arr, r, 'check if Uint8Array is correct');
    });
    it('should return a hex', function () {
        const r = arr.toHex();
        assert_1.default.deepEqual(hex, r, 'check if hex is correct');
    });
    it('should return a subarray', function () {
        const r = arr.subarray(1, 3);
        assert_1.default.deepEqual(new Uint8Array([2, 3]), r, 'check if subarray is correct');
    });
});
//# sourceMappingURL=uint8ArrayE.spec.js.map