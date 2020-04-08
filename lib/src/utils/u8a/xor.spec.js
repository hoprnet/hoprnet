"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const xor_1 = require("./xor");
describe('testing XORing Uint8Array', function () {
    it('should XOR two arrays', function () {
        let a = new Uint8Array([0, 255, 0, 255, 0]);
        let b = new Uint8Array([255, 0, 255, 0, 255]);
        let aXORb = new Uint8Array([255, 255, 255, 255, 255]);
        assert_1.default.deepEqual(xor_1.u8aXOR(false, a, b), aXORb);
        xor_1.u8aXOR(true, a, b);
        assert_1.default.deepEqual(a, aXORb);
    });
    it('should XOR more than two arrays', function () {
        let a = new Uint8Array([0, 255, 0, 255, 0]);
        let b = new Uint8Array([255, 0, 255, 0, 255]);
        let c = new Uint8Array([0, 0, 255, 0, 0]);
        let aXORbXORc = new Uint8Array([255, 255, 0, 255, 255]);
        assert_1.default.deepEqual(xor_1.u8aXOR(false, a, b, c), aXORbXORc);
        xor_1.u8aXOR(true, a, b, c);
        assert_1.default.deepEqual(a, aXORbXORc);
    });
});
