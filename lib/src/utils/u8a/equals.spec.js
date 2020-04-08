"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const equals_1 = require("./equals");
const crypto_1 = require("crypto");
describe('test u8a equals', function () {
    it('should check whether two (or more) Uint8Arrays are equal', function () {
        assert_1.default(equals_1.u8aEquals(new Uint8Array(), new Uint8Array()), `check empty array`);
        assert_1.default(!equals_1.u8aEquals(crypto_1.randomBytes(32), crypto_1.randomBytes(32)), `random data should be with high probability not equal`);
        assert_1.default(!equals_1.u8aEquals(crypto_1.randomBytes(32), crypto_1.randomBytes(31)), `random data should be with high probability not equal, different size`);
        assert_1.default(equals_1.u8aEquals(new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xff)), `check equal arrays`);
        assert_1.default(!equals_1.u8aEquals(new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xff), new Uint8Array(32).fill(0xaa)), `check different arrays`);
        assert_1.default(!equals_1.u8aEquals(crypto_1.randomBytes(32), crypto_1.randomBytes(32), crypto_1.randomBytes(32)), `random data should be with high probability not equal`);
        // @ts-ignore
        assert_1.default.throws(() => equals_1.u8aEquals(new Uint8Array(), undefined), `check undefined b`);
        // @ts-ignore
        assert_1.default.throws(() => equals_1.u8aEquals(new Uint8Array(), new Uint8Array(), undefined), `check undefined rest`);
    });
});
