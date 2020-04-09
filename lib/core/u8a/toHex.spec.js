"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const toHex_1 = require("./toHex");
const assert_1 = __importDefault(require("assert"));
describe('test toHex', function () {
    it('should create a Hex string', function () {
        assert_1.default(toHex_1.u8aToHex(new Uint8Array([])) == '0x');
        assert_1.default(toHex_1.u8aToHex(new Uint8Array([]), false) == '');
        assert_1.default(toHex_1.u8aToHex(new Uint8Array([1, 2, 3])) == '0x010203');
        assert_1.default(toHex_1.u8aToHex(new Uint8Array([1, 2, 3]), false) == '010203');
    });
});
