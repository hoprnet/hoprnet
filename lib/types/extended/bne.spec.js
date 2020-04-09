"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const assert_1 = __importDefault(require("assert"));
const bne_1 = __importDefault(require("./bne"));
describe('test BNE', function () {
    it('should returns a Uint8Array', function () {
        const number = 1;
        assert_1.default.deepEqual(new bne_1.default(number).toU8a(), new Uint8Array([number]), 'check if BNE u8a array is correct');
    });
});
