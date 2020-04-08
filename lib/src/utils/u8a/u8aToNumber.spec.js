"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const u8aToNumber_1 = require("./u8aToNumber");
describe('test u8aToNumber', function () {
    it('should convert a u8a to a number', function () {
        assert_1.default(u8aToNumber_1.u8aToNumber(new Uint8Array()) == 0);
        assert_1.default(u8aToNumber_1.u8aToNumber(new Uint8Array([1])) == 1);
        assert_1.default(u8aToNumber_1.u8aToNumber(new Uint8Array([1, 0])) == 256);
        assert_1.default(u8aToNumber_1.u8aToNumber(new Uint8Array([1, 1])) == 257);
    });
});
